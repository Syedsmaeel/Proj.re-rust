use anyhow::{Result, Context};
use candle_core::{Device, Tensor};
use candle_core::quantized::gguf_file;
use candle_transformers::models::quantized_phi3::ModelWeights as Phi3;
use tokenizers::Tokenizer;

pub struct GgufBackend {
    model: Phi3,
    tokenizer: Tokenizer,
    device: Device,
}

impl GgufBackend {
    pub fn load(gguf_path: &str, tokenizer_path: &str) -> Result<Self> {
        let device = Device::Cpu;
        let mut file = std::fs::File::open(gguf_path)?;
        let content = gguf_file::Content::read(&mut file)?;
        let model = Phi3::from_gguf(content, &mut file, &device)?;
        let tokenizer = Tokenizer::from_file(tokenizer_path).map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok(Self { model, tokenizer, device })
    }
}
