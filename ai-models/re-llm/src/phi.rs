//! Microsoft Phi-3 inference using `candle`.
//!
//! Implements the loader + autoregressive generation loop. Public surface is
//! intentionally narrow: load a model, generate from a prompt. The OpenClaw
//! port (`re-agent`) calls into this from its Send command.

use anyhow::{anyhow, Context, Result};
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::phi3::{Config as Phi3Config, Model as Phi3};
use hf_hub::api::sync::{Api, ApiRepo};
use std::path::PathBuf;
use tokenizers::Tokenizer;

/// Which Phi-3 variant to load. Defaults to `MiniInstruct4k` (~3.8B params,
/// 4k context, runs on CPU in ~8 GB RAM, on a modest GPU much faster).
#[derive(Debug, Clone, Copy)]
pub enum PhiVariant {
    /// `microsoft/Phi-3-mini-4k-instruct` — recommended starting point.
    MiniInstruct4k,
    /// `microsoft/Phi-3-mini-128k-instruct` — long-context variant.
    MiniInstruct128k,
    /// `microsoft/Phi-3-medium-4k-instruct` — 14B, needs more RAM/VRAM.
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

/// Generation hyperparameters.
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

/// Loaded Phi-3 model + tokenizer, ready for repeated generation calls.
pub struct PhiBackend {
    model: Phi3,
    tokenizer: Tokenizer,
    device: Device,
    eos_token: u32,
}

impl PhiBackend {
    /// Download (cached) and load a Phi-3 variant from Hugging Face.
    ///
    /// Note: first run will pull several GB of weights into the HF cache.
    pub fn load(variant: PhiVariant) -> Result<Self> {
        let device = pick_device();
        let api = Api::new().context("hf-hub api init failed")?;
        let repo = api.model(variant.repo_id().to_string());

        let tokenizer_path = repo
            .get("tokenizer.json")
            .context("download tokenizer.json")?;
        let config_path = repo.get("config.json").context("download config.json")?;
        let weights = collect_safetensors(&repo)?;

        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow!("load tokenizer: {e}"))?;

        let config_str = std::fs::read_to_string(&config_path)?;
        let config: Phi3Config = serde_json::from_str(&config_str)
            .context("parse Phi-3 config.json")?;

        // Phi-3 weights are float16 on the hub; load matching dtype.
        let dtype = if device.is_cuda() { DType::BF16 } else { DType::F32 };
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&weights, dtype, &device)? };
        let model = Phi3::new(&config, vb)?;

        let eos_token = tokenizer
            .token_to_id("<|end|>")
            .or_else(|| tokenizer.token_to_id("<|endoftext|>"))
            .unwrap_or(0);

        Ok(Self {
            model,
            tokenizer,
            device,
            eos_token,
        })
    }

    /// Run autoregressive generation against a single user prompt.
    /// Wraps the prompt in Phi-3's chat template.
    pub fn generate(&mut self, prompt: &str, cfg: &GenerationConfig) -> Result<String> {
        let formatted = format_phi3_chat(prompt);

        let encoding = self
            .tokenizer
            .encode(formatted, true)
            .map_err(|e| anyhow!("tokenize prompt: {e}"))?;
        let mut tokens: Vec<u32> = encoding.get_ids().to_vec();

        let mut logits_processor =
            LogitsProcessor::new(cfg.seed, Some(cfg.temperature), cfg.top_p);

        let mut output = String::new();
        let mut pos: usize = 0;

        for step in 0..cfg.max_tokens {
            // First step ingests the whole prompt; subsequent steps feed only
            // the newest token, relying on the kv-cache inside the model.
            let context_size = if step == 0 { tokens.len() } else { 1 };
            let start = tokens.len().saturating_sub(context_size);
            let ctx_slice = &tokens[start..];

            let input = Tensor::new(ctx_slice, &self.device)?.unsqueeze(0)?;
            let logits = self.model.forward(&input, pos)?;
            let logits = logits.squeeze(0)?.to_dtype(DType::F32)?;
            pos += ctx_slice.len();

            let next = logits_processor.sample(&logits)?;
            tokens.push(next);

            if next == self.eos_token {
                break;
            }

            if let Ok(piece) = self.tokenizer.decode(&[next], true) {
                output.push_str(&piece);
            }
        }

        Ok(output)
    }
}

/// Build the file list for a model whose weights are sharded across multiple
/// safetensors files (the common Phi-3 layout).
fn collect_safetensors(repo: &ApiRepo) -> Result<Vec<PathBuf>> {
    if let Ok(index_path) = repo.get("model.safetensors.index.json") {
        let txt = std::fs::read_to_string(&index_path)?;
        let v: serde_json::Value = serde_json::from_str(&txt)?;
        let weight_map = v
            .get("weight_map")
            .and_then(|w| w.as_object())
            .ok_or_else(|| anyhow!("missing weight_map in safetensors index"))?;

        let mut files: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for (_, val) in weight_map {
            if let Some(s) = val.as_str() {
                files.insert(s.to_string());
            }
        }

        let mut paths = Vec::new();
        for f in files {
            paths.push(repo.get(&f).with_context(|| format!("download shard {f}"))?);
        }
        Ok(paths)
    } else {
        let p = repo
            .get("model.safetensors")
            .context("download model.safetensors")?;
        Ok(vec![p])
    }
}

fn pick_device() -> Device {
    #[cfg(feature = "cuda")]
    if let Ok(d) = Device::new_cuda(0) {
        return d;
    }
    #[cfg(feature = "metal")]
    if let Ok(d) = Device::new_metal(0) {
        return d;
    }
    Device::Cpu
}

/// Phi-3 chat template — the model expects this exact framing.
fn format_phi3_chat(user: &str) -> String {
    format!("<|user|>\n{user}<|end|>\n<|assistant|>\n")
}
