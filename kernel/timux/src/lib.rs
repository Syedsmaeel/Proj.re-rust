//! Timux — Sovereign Kernel
//!
//! Hybrid privilege model: custom rings + capability-based access control.
//!
//! ## Architecture
//! - `priv`  — ring definitions, ring transitions, privilege enforcement
//! - `cap`   — capability tokens, capability tables, delegation
//! - `mm`    — memory management, page tables, virtual memory
//! - `sched` — task scheduler, context switching
//! - `ipc`   — inter-process communication via capability channels
//! - `arch`  — architecture-specific implementations (x86_64, riscv, arm64)
//! - `boot`  — boot protocol, early init

#![no_std]
#![allow(dead_code)]

extern crate alloc;

pub mod arch;
pub mod boot;
pub mod cap;
pub mod ipc;
pub mod mm;
pub mod priv_model;
pub mod sched;

pub use priv_model::{Ring, RingLevel, Capability, CapabilityToken};
