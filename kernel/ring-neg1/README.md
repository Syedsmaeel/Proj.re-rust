# Ring -1 — Timux Sovereign Bootloader

Ring -1 sits below Ring 0. It owns the machine before the kernel does.

## Boot paths

| Mode | Binary | How |
|---|---|---|
| UEFI (primary) | `EFI/BOOT/BOOTX64.EFI` | Firmware loads EFI app directly |
| BIOS (fallback) | MBR stage1 → stage2 | INT 0x19 → 512B stage1 → stage2 |

## What Ring -1 does

1. **Detect** boot mode (UEFI or BIOS)
2. **Parse** memory map (UEFI GetMemoryMap or BIOS E820)
3. **Initialize** framebuffer (GOP on UEFI, VESA on BIOS)
4. **Render** Bash++ TUI boot menu to framebuffer (pixel-level, no font lib)
5. **Wait** for user input or auto-boot after timeout
6. **Load** selected Timux kernel ELF from disk
7. **Build** HandoffInfo struct (memory map, framebuffer, cmdline, heap location)
8. **Exit** boot services (UEFI) / disable interrupts (BIOS)
9. **Jump** to Timux `_start` with `HandoffInfo*` in RDI

## HandoffInfo contract

Ring -1 fills a `HandoffInfo` struct and passes it to Timux via RDI:

```rust
pub struct HandoffInfo {
    pub magic:        u64,          // 0x54494D55585F424F — Timux verifies this
    pub protocol:     BootProtocol, // Uefi or Bios
    pub kernel_entry: u64,          // virtual entry point
    pub heap_start:   u64,          // largest usable RAM region
    pub heap_size:    u64,
    pub fb:           FramebufferInfo,
    pub cmdline:      [u8; 256],
    pub mmap_count:   u32,
    pub mmap:         [MmapEntry; 128],
    pub loader_start: u64,          // Ring -1 memory — Timux can reclaim
    pub loader_size:  u64,
    pub rsdp_addr:    u64,          // ACPI root table
}
```

## Boot menu entries (default)

| Entry | Cmdline | Description |
|---|---|---|
| Timux (default) | `ring0 loglevel=3` | Normal boot |
| Timux (recovery) | `ring0 recovery=1 loglevel=5` | Recovery mode |
| Timux (debug) | `ring0 debug=1 serial=1` | Debug + serial output |
| Bash++ shell only | `ring2 bashpp=1 shell_only=1` | Boot straight to Bash++ TUI |
| Memory test | — | Hardware memory test |

## ISO structure

```
timux.iso
├── boot/
│   ├── grub/
│   │   └── grub.cfg     ← BIOS fallback menu
│   └── timux.elf        ← Timux kernel
└── EFI/
    └── BOOT/
        └── BOOTX64.EFI  ← Ring -1 UEFI binary (primary)
```

## Build

```bash
# Install tools
sudo apt install grub-pc-bin grub-efi-amd64-bin xorriso mtools

# Build ISO
./build.sh

# Test (BIOS)
qemu-system-x86_64 -cdrom timux.iso -m 512M -serial stdio

# Test (UEFI)
qemu-system-x86_64 -cdrom timux.iso -m 512M \
    -bios /usr/share/ovmf/OVMF.fd
```

## Architecture

```
ring-neg1/
├── src/
│   ├── lib.rs       — crate root
│   ├── mem/         — memory map (UEFI + E820)
│   ├── display/     — framebuffer pixel renderer
│   ├── menu/        — Bash++ TUI boot menu (pixel rendering)
│   ├── proto/       — HandoffInfo contract (Ring -1 → Ring 0)
│   ├── uefi/        — UEFI boot path + GOP
│   └── bios/        — BIOS boot path + E820 + VESA
├── iso/
│   ├── boot/grub/grub.cfg
│   └── EFI/BOOT/    ← BOOTX64.EFI placed here after build
└── build.sh         — ISO builder script
```

AGPL-3.0 — Syed Ismaeel, Lucknow Est. 2019
