# 🌊 qual-sea

**qual-sea** is the elite quantization and weight-compression engine for the **Proj.re-rust** Sovereignty Stack. It transforms heavy, resource-intensive AI models into lean, memory-mappable GGUF binaries.

Built in pure Rust, `qual-sea` provides the tools necessary to achieve local AI sovereignty on modest hardware by shrinking model footprints without sacrificing intelligence.

## 🚀 Key Features

-   **Multi-threaded Compression:** Powered by `rayon` to utilize every CPU core for maximum throughput.
-   **Advanced K-Quants:** Supports industry-standard K-Quants (`Q4_K_M`, `Q5_K_M`, etc.) for superior "intelligence-per-bit" compared to legacy 4-bit methods.
-   **GGUF Export:** Generates GGUF archives ready for zero-copy memory mapping (mmap) in `re-llm`.
-   **SafeTensors Integration:** Direct ingestion of Hugging Face SafeTensors formats.
-   **Zero-Dependency Runtime:** Compiles to a single standalone binary.

## 🛠️ Usage

### Compress a model
To compress a standard 15GB Phi-3 model into a ~2.2GB elite GGUF:

```bash
cargo run --release -p qual-sea -- compress \
  --input path/to/model.safetensors \
  --output models/phi3-mini-q4km.gguf \
  --method q4-k-m
```

### Supported Methods
| Method | Description | Target Use Case |
| :--- | :--- | :--- |
| `q4-k-m` | 4-bit Medium | **Recommended.** Best balance of speed and logic. |
| `q4-k-s` | 4-bit Small | Minimum RAM footprint for Phi-3. |
| `q5-k-m` | 5-bit Medium | High-precision for complex reasoning. |
| `q8-0` | 8-bit | Near-lossless precision. |
| `q4-0` | Legacy 4-bit | Fastest inference on older CPUs. |

## 🏗️ Architecture

`qual-sea` operates as a parallel pipeline:
1.  **Ingestion:** Maps SafeTensors into local memory.
2.  **Analysis:** Identifies layer types (Attention, MLP, Embeddings).
3.  **Parallel Quantization:** Spawns worker threads to compress layers concurrently.
4.  **Serialization:** Packs quantized blocks into a structured GGUF file.

## 📜 License
Part of the Proj.re-rust stack. Licensed under **AGPL-3.0**. 

---
*Syed Ismaeel — Lucknow, Est. 2019*
