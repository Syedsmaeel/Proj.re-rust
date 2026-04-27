use anyhow::{Result, Context};
use clap::{Parser, Subcommand};
use secrecy::{SecretString, ExposeSecret};
use std::fs::File;
use std::io::{Read, Write};

#[derive(Parser)]
#[command(name = "re-crypt", about = "Sovereign Security Toolkit", version)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Encrypt a file using a passphrase (age format)
    Encrypt {
        input: String,
        #[arg(short, long)]
        output: String,
    },
    /// Decrypt a file using a passphrase
    Decrypt {
        input: String,
        #[arg(short, long)]
        output: String,
    },
    /// Hash a string using Argon2id (best for passwords)
    Hash {
        text: String,
    },
    /// Generate a cryptographically secure random secret
    Gen {
        /// Length in bytes
        #[arg(short, long, default_value_t = 32)]
        length: usize,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.cmd {
        Cmd::Encrypt { input, output } => {
            println!("🔒 re-crypt — Encrypting {}...", input);
            let password = rpassword::prompt_password("Enter Passphrase: ")?;
            let mut input_file = File::open(input)?;
            let mut data = Vec::new();
            input_file.read_to_end(&mut data)?;

            let encryptor = age::Encryptor::with_user_passphrase(SecretString::new(password));
            let mut output_file = File::create(output)?;
            let mut writer = encryptor.wrap_output(&mut output_file)?;
            writer.write_all(&data)?;
            writer.finish()?;
            println!("✅ File encrypted successfully.");
        }
        
        Cmd::Decrypt { input, output } => {
            println!("🔓 re-crypt — Decrypting {}...", input);
            let password = rpassword::prompt_password("Enter Passphrase: ")?;
            let input_file = File::open(input)?;
            
            let decryptor = match age::Decryptor::new(input_file)? {
                age::Decryptor::Passphrase(d) => d,
                _ => anyhow::bail!("Unsupported encryption format"),
            };

            let mut reader = decryptor.decrypt(&SecretString::new(password), None)?;
            let mut data = Vec::new();
            reader.read_to_end(&mut data)?;
            
            let mut output_file = File::create(output)?;
            output_file.write_all(&data)?;
            println!("✅ File decrypted to {}.", output);
        }

        Cmd::Hash { text } => {
            use argon2::{password_hash::{PasswordHasher, SaltString}, Argon2};
            println!("🛡️ re-crypt — Hashing via Argon2id...");
            let salt = SaltString::generate(&mut rand::thread_rng());
            let argon2 = Argon2::default();
            let hash = argon2.hash_password(text.as_bytes(), &salt)
                .map_err(|e| anyhow::anyhow!("Hashing failed: {}", e))?;
            println!("  Hash: {}", hash);
        }

        Cmd::Gen { length } => {
            use rand::RngCore;
            let mut buf = vec![0u8; length];
            rand::thread_rng().fill_bytes(&mut buf);
            println!("🔑 re-crypt — New Secret (Hex):");
            println!("  {}", hex::encode(buf));
        }
    }
    
    Ok(())
}
