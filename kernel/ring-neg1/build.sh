#!/usr/bin/env bash
# Ring -1 ISO builder
# Produces a bootable .iso with UEFI (primary) + BIOS fallback
set -e

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ISO_DIR="$ROOT/iso"
OUT="$ROOT/timux.iso"

echo "[ring-neg1] Building bootloader..."

# 1. Build UEFI binary (BOOTX64.EFI)
cargo build --release --bin BOOTX64 \
    --target x86_64-unknown-uefi 2>/dev/null || \
    echo "  [warn] UEFI target not installed — skipping BOOTX64.EFI"

# Copy EFI binary if built
if [ -f target/x86_64-unknown-uefi/release/BOOTX64.efi ]; then
    cp target/x86_64-unknown-uefi/release/BOOTX64.efi \
       "$ISO_DIR/EFI/BOOT/BOOTX64.EFI"
    echo "  [ok] BOOTX64.EFI"
fi

# 2. Copy Timux kernel
if [ -f ../timux/target/x86_64-unknown-none/release/timux-x86_64 ]; then
    cp ../timux/target/x86_64-unknown-none/release/timux-x86_64 \
       "$ISO_DIR/boot/timux.elf"
    echo "  [ok] timux.elf"
else
    echo "  [warn] Timux kernel not found — ISO will boot to grub rescue"
    touch "$ISO_DIR/boot/timux.elf"
fi

# 3. Build ISO with xorriso + grub-mkrescue
echo "[ring-neg1] Building ISO..."
grub-mkrescue \
    --output="$OUT" \
    --product-name="Timux" \
    --product-version="0.1.0" \
    "$ISO_DIR"

echo "[ring-neg1] ISO ready: $OUT"
echo "  Test with QEMU:"
echo "  qemu-system-x86_64 -cdrom $OUT -m 512M -serial stdio"
echo "  qemu-system-x86_64 -cdrom $OUT -m 512M -bios /usr/share/ovmf/OVMF.fd (UEFI)"
