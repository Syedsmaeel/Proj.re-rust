use anyhow::{anyhow, Context, Result};
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::phi3::{Config as Phi3Config, Model as Phi3};
use hf_hub::api::sync::{Api, ApiRepo};
use std::path::PathBuf;
use tokenizers::Tokenizer;

#[derive(Debug, Clone, Copy)]
pub enum PhiVariant {
    MiniInstruct4k,
    MiniInstruct128k,
    MediumInstruct4k,
}

impl PhiVariant {
    pub fn repo_id(&self) -> &'static str {
        match self {
            PhiVariant::MiniInstruct4k => "microsoft/Phi-3-mini-4k-instruct",
            PhiVariant::MiniInstruct128k => "microsoft/Phi-3-mini-128k-instruct",
            PhiVariant::MediumInstruct4k => "microsoft/Phi-3-medium-4k-instruct",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "mini-128k" | "phi-3-mini-128k" => PhiVariant::MiniInstruct128k,
            "medium" | "medium-4k" | "phi-3-medium" => PhiVariant::MediumInstruct4k,
            _ => PhiVariant::MiniInstruct4k,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GenerationConfig {
    pub max_tokens: usize,
    pub temperature: f64,
    pub top_p: Option<f64>,
    pub seed: u64,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            max_tokens: 256,
            temperature: 0.7,
            top_p: Some(0.9),
            seed: 42,
        }
    }
}

pub struct PhiBackend {
    model: Phi3,
    tokenizer: Tokenizer,
    device: Device,
    eos_token: u32,
}

impl PhiBackend {
    pub fn load(variant: PhiVariant) -> Result<Self> {
        let device = pick_device();
        let api = Api::new().context("hf-hub api init failed")?;
        let repo = api.model(variant.repo_id().to_string());

        let tokenizer_path = repo.get("tokenizer.json")?;
        let config_path = repo.get("config.json")?;
        let weights = collect_safetensors(&repo)?;

        let tokenizer = Tokenizer::from_file(&tokenizer_path).map_err(|e| anyhow!("{e}"))?;
        let config_str = std::fs::read_to_string(&config_path)?;
        let config: Phi3Config = serde_json::from_str(&config_str)?;

        let dtype = if device.is_cuda() { DType::BF16 } else { DType::F32 };
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&weights, dtype, &device)? };
        let model = Phi3::new(&config, vb)?;

        let eos_token = tokenizer.token_to_id("<|end|>")
            .or_else(|| tokenizer.token_to_id("<|endoftext|>"))
            .unwrap_or(0);

        Ok(Self { model, tokenizer, device, eos_token })
    }

    /// Generate with a callback for each new token (streaming).
    pub fn generate_streaming<F>(&mut self, system: Option<&str>, prompt: &str, cfg: &GenerationConfig, mut callback: F) -> Result<String>
    where F: FnMut(&str) -> Result<()> 
    {
        let formatted = format_phi3_chat(system, prompt);
        let encoding = self.tokenizer.encode(formatted, true).map_err(|e| anyhow!("{e}"))?;
        let mut tokens: Vec<u32> = encoding.get_ids().to_vec();
        let mut logits_processor = LogitsProcessor::new(cfg.seed, Some(cfg.temperature), cfg.top_p);

        let mut full_output = String::new();
        let mut pos: usize = 0;

        for step in 0..cfg.max_tokens {
            let context_size = if step == 0 { tokens.len() } else { 1 };
            let start = tokens.len().saturating_sub(context_size);
            let ctx_slice = &tokens[start..];

            let input = Tensor::new(ctx_slice, &self.device)?.unsqueeze(0)?;
            let logits = self.model.forward(&input, pos)?;
            let logits = logits.squeeze(0)?.to_dtype(DType::F32)?;
            pos += ctx_slice.len();

            let next = logits_processor.sample(&logits)?;
            tokens.push(next);

            if next == self.eos_token { break; }

            if let Ok(piece) = self.tokenizer.decode(&[next], true) {
                full_output.push_str(&piece);
                callback(&piece)?;
            }
        }

        Ok(full_output)
    }

    /// Legacy non-streaming wrapper.
    pub fn generate(&mut self, prompt: &str, cfg: &GenerationConfig) -> Result<String> {
        self.generate_streaming(None, prompt, cfg, |_| Ok(()))
    }
}

fn collect_safetensors(repo: &ApiRepo) -> Result<Vec<PathBuf>> {
    if let Ok(index_path) = repo.get("model.safetensors.index.json") {
        let txt = std::fs::read_to_string(&index_path)?;
        let v: serde_json::Value = serde_json::from_str(&txt)?;
        let weight_map = v.get("weight_map").and_then(|w| w.as_object()).ok_or_else(|| anyhow!("no weight_map"))?;
        let mut files = std::collections::BTreeSet::new();
        for (_, val) in weight_map { if let Some(s) = val.as_str() { files.insert(s.to_string()); } }
        let mut paths = Vec::new();
        for f in files { paths.push(repo.get(&f)?); }
        Ok(paths)
    } else {
        Ok(vec![repo.get("model.safetensors")?])
    }
}

fn pick_device() -> Device {
    #[cfg(feature = "cuda")] if let Ok(d) = Device::new_cuda(0) { return d; }
    #[cfg(feature = "metal")] if let Ok(d) = Device::new_metal(0) { return d; }
    Device::Cpu
}

fn format_phi3_chat(system: Option<&str>, user: &str) -> String {
    match system {
        Some(s) => format!("<|system|>\n{s}<|end|>\n<|user|>\n{user}<|end|>\n<|assistant|>\n"),
        None => format!("<|user|>\n{user}<|end|>\n<|assistant|>\n"),
    }
}
