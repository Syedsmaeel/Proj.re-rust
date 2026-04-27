use anyhow::{Result, Context};
use candle_core::{Device, Tensor};
use candle_quants::{GgmlType, precomputed_quantized_tensor, quantized_var_builder::save_gguf};
use clap::{Parser, Subcommand, ValueEnum};
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Parser)]
#[command(name = "qual-sea", about = "Elite Quantization Engine — Sovereignty Stack", version)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Compress model weights using advanced K-Quants
    Compress {
        /// Input safetensors file
        #[arg(short, long)]
        input: String,
        /// Output .gguf file
        #[arg(short, long)]
        output: String,
        /// Quantization method (K-Quants provide better intelligence/size ratio)
        #[arg(short, long, value_enum, default_value = "q4-k-m")]
        method: Method,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Method {
    Q4_0,
    Q4_1,
    Q5_0,
    Q5_1,
    Q8_0,
    /// 4-bit Medium (Best balance)
    Q4KM,
    /// 4-bit Small (Lower RAM)
    Q4KS,
    /// 5-bit Medium (High intelligence)
    Q5KM,
}

impl Method {
    fn to_ggml(&self) -> GgmlType {
        match self {
            Method::Q4_0 => GgmlType::Q4_0,
            Method::Q4_1 => GgmlType::Q4_1,
            Method::Q5_0 => GgmlType::Q5_0,
            Method::Q5_1 => GgmlType::Q5_1,
            Method::Q8_0 => GgmlType::Q8_0,
            Method::Q4KM => GgmlType::Q4_K_M,
            Method::Q4KS => GgmlType::Q4_K_S,
            Method::Q5KM => GgmlType::Q5_K_M,
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Compress { input, output, method } => {
            compress_model(&input, &output, method)?;
        }
    }
    Ok(())
}

fn compress_model(input_path: &str, output_path: &str, method: Method) -> Result<()> {
    let device = Device::Cpu;
    println!("🌊 qual-sea — Parallel Compression Engine Active");
    
    let tensors = candle_core::safetensors::load(input_path, &device)
        .context("Failed to load safetensors")?;
    
    let total = tensors.len();
    println!("🌊 qual-sea — Found {} tensors. Quantizing via {}...", total, method.to_possible_value().unwrap().get_name());

    // Use Arc<Mutex<...>> to collect results across threads safely
    let quantized_tensors = Arc::new(Mutex::new(HashMap::new()));
    let ggml_method = method.to_ggml();

    // Parallelize the quantization loop using Rayon
    tensors.into_iter().collect::<Vec<_>>().into_par_iter().for_each(|(name, tensor)| {
        let is_weight = name.contains("weight") && tensor.rank() >= 2;
        
        let result = if is_weight {
            println!("  ⚡ [Quantizing] {}", name);
            precomputed_quantized_tensor(&tensor, ggml_method)
        } else {
            println!("  ✨ [Passing   ] {}", name);
            precomputed_quantized_tensor(&tensor, GgmlType::F32)
        };

        if let Ok(q_tensor) = result {
            quantized_tensors.lock().unwrap().insert(name, q_tensor);
        }
    });

    println!("\n🌊 qual-sea — Finalizing GGUF archive...");
    let final_map = Arc::try_unwrap(quantized_tensors).unwrap().into_inner().unwrap();
    
    save_gguf(Path::new(output_path), &final_map, &HashMap::new())
        .context("Failed to save GGUF file")?;

    println!("✅ Successfully compressed with {} threading!", rayon::current_num_threads());
    Ok(())
}
