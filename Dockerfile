FROM rust:1.75-bookworm

# Install dependencies for TBM development and QEMU testing
RUN apt-get update && apt-get install -y \
    qemu-system-x86 \
    ovmf \
    xorriso \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install rustup components for bare-metal targets
RUN rustup target add x86_64-unknown-uefi x86_64-unknown-none

# Set up the workspace
WORKDIR /app

# The command to build the bootloader
CMD ["cargo", "build", "-p", "tbm", "--target", "x86_64-unknown-uefi"]
