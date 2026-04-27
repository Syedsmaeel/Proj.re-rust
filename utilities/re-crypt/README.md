# 🛡️ re-crypt

**re-crypt** is the Sovereign Security & Encryption Toolkit for the **Proj.re-rust** stack. It provides modern, high-entropy cryptographic primitives for securing data, identities, and communications.

## 🚀 Key Features

-   **Passphrase Encryption:** Uses the `age` format (X25519) for simple, secure file encryption that replaces legacy PGP.
-   **Elite Password Hashing:** Implements `Argon2id`, the cryptographically recommended standard for protecting user credentials.
-   **Cryptographic Entropy:** Secure generation of high-randomness secrets and tokens using `rand`.
-   **Memory Safety:** Leverages the `secrecy` crate to ensure sensitive keys are zeroed out in memory and protected from accidental logging.

## 🛠️ Usage

### Encrypt a file
```bash
cargo run -p re-crypt -- encrypt sensitive.txt --output sensitive.age
```

### Hash a password (Argon2id)
```bash
cargo run -p re-crypt -- hash "your-secure-password"
```

### Generate a 64-byte secret key
```bash
cargo run -p re-crypt -- gen --length 64
```

## 🏗️ Technical Stack

-   **Encryption:** `age` (Rust implementation)
-   **Hashing:** `argon2` (Rust implementation)
-   **Randomness:** `rand_chacha`
-   **Encoding:** `hex`

## 📜 License
Part of the Proj.re-rust stack. Licensed under **AGPL-3.0**. 

---
*Syed Ismaeel — Lucknow, Est. 2019*
