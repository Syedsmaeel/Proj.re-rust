use anyhow::{anyhow, Context, Result};
use candle_core::{Device, Tensor};
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::quantized_phi3::ModelWeights as Phi3;
use candle_quants::gguf_file;
use tokenizers::Tokenizer;
use std::fs::File;

pub struct GgufBackend {
    model: Phi3,
    tokenizer: Tokenizer,
    device: Device,
}

impl GgufBackend {
    pub fn load(gguf_path: &str, tokenizer_path: &str) -> Result<Self> {
        let device = Device::Cpu;
        let mut file = File::open(gguf_path).context("Failed to open GGUF file")?;
        let content = gguf_file::Content::read(&mut file).context("Read GGUF content")?;
        let model = Phi3::from_gguf(content, &mut file, &device).context("Load Phi3 from GGUF")?;
        
        let tokenizer = Tokenizer::from_file(tokenizer_path).map_err(|e| anyhow!("{e}"))?;

        Ok(Self { model, tokenizer, device })
    }

    pub fn generate_streaming<F>(&mut self, prompt: &str, max_tokens: usize, temp: f64, mut callback: F) -> Result<String>
    where F: FnMut(&str) -> Result<()> 
    {
        // Simple chat template for Phi-3
        let formatted = format!("<|user|>\n{prompt}<|end|>\n<|assistant|>\n");
        let encoding = self.tokenizer.encode(formatted, true).map_err(|e| anyhow!("{e}"))?;
        let mut tokens = encoding.get_ids().to_vec();
        let mut logits_processor = LogitsProcessor::new(42, Some(temp), None);

        let mut full_output = String::new();
        
        for step in 0..max_tokens {
            let context_size = if step == 0 { tokens.len() } else { 1 };
            let start = tokens.len().saturating_sub(context_size);
            let input = Tensor::new(&tokens[start..], &self.device)?.unsqueeze(0)?;
            
            let logits = self.model.forward(&input, tokens.len() - context_size)?;
            let logits = logits.squeeze(0)?;
            
            let next = logits_processor.sample(&logits)?;
            tokens.push(next);

            if let Ok(piece) = self.tokenizer.decode(&[next], true) {
                if piece.contains("<|end|>") { break; }
                full_output.push_str(&piece);
                callback(&piece)?;
            }
        }

        Ok(full_output)
    }
}
