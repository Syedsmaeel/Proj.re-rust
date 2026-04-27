use anyhow::{Result, Context};
use candle_core::{Device, Tensor, DType};
use candle_quants::quantized_var_builder::save_gguf;
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::path::Path;

#[derive(Parser)]
#[command(name = "qual-sea", about = "Proj.re-rust Quantization Engine", version)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Quantize a model from Safetensors to GGUF format
    Compress {
        /// Input safetensors file
        #[arg(short, long)]
        input: String,
        /// Output .gguf file
        #[arg(short, long)]
        output: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.cmd {
        Cmd::Compress { input, output } => {
            compress_model(&input, &output)?;
        }
    }
    
    Ok(())
}

fn compress_model(input_path: &str, output_path: &str) -> Result<()> {
    let device = Device::Cpu;
    println!("🌊 qual-sea — Loading tensors from {}...", input_path);
    
    let tensors = candle_core::safetensors::load(input_path, &device)
        .context("Failed to load safetensors. Make sure the path is correct.")?;
    
    println!("🌊 qual-sea — Found {} tensors. Starting quantization...", tensors.len());
    
    let mut quantized_tensors = HashMap::new();
    
    for (name, tensor) in tensors.iter() {
        // We only quantize weights of linear and embedding layers.
        // Biases, layer-norms, and small vectors are kept in F32/F16 for accuracy.
        let should_quantize = name.contains("weight") && tensor.rank() >= 2;
        
        if should_quantize {
            print!("  [Q4_0] Quantizing {}... ", name);
            // Convert to 4-bit (Q4_0)
            let q_tensor = candle_quants::precomputed_quantized_tensor(tensor, candle_quants::GgmlType::Q4_0)?;
            quantized_tensors.insert(name.clone(), q_tensor);
            println!("done.");
        } else {
            println!("  [F32 ] Passing through {}...", name);
            // Wrap F32 tensor as "quantized" (no actual bits lost)
            let q_tensor = candle_quants::precomputed_quantized_tensor(tensor, candle_quants::GgmlType::F32)?;
            quantized_tensors.insert(name.clone(), q_tensor);
        }
    }

    println!("🌊 qual-sea — Writing compressed model to {}...", output_path);
    
    // Save as GGUF so it can be memory-mapped (mmap) later for 50MB-style usage
    save_gguf(Path::new(output_path), &quantized_tensors, &HashMap::new())
        .context("Failed to save GGUF file")?;

    println!("\n✅ Successfully compressed model! You can now load this in re-llm.");
    Ok(())
}
