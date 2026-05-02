//! Ring -1 — Timux Sovereign Bootloader
//!
//! Ring -1 sits below Ring 0. It owns the machine before the kernel does.
//!
//! ## Boot sequence
//!
//! UEFI path (primary):
//!   UEFI firmware → BOOTX64.EFI → memory map → framebuffer → TUI menu → Timux
//!
//! BIOS path (fallback):
//!   BIOS → MBR (stage1, 512B asm) → stage2 → protected mode → long mode → Timux
//!
//! ## What Ring -1 does
//!   1. Detect boot mode (UEFI or BIOS)
//!   2. Query and parse memory map
//!   3. Initialize framebuffer (GOP on UEFI, VESA on BIOS)
//!   4. Render Bash++ TUI boot menu
//!   5. Load Timux kernel ELF from disk
//!   6. Build BootInfo struct
//!   7. Exit boot services (UEFI) / disable interrupts (BIOS)
//!   8. Jump to Timux _start

#![no_std]
#![allow(dead_code)]

extern crate alloc;

pub mod display;
pub mod mem;
pub mod menu;
pub mod proto;
pub mod uefi;
pub mod bios;

pub use mem::{MemoryMap, MemoryRegion, MemoryKind};
pub use display::{Framebuffer, Color, Pixel};
pub use menu::{BootMenu, BootEntry, MenuResult};
pub use proto::{BootProtocol, HandoffInfo};
