//! Timux — Sovereign Kernel
//!
//! Hybrid privilege model: 5 custom rings + capability-based access control.
//! Sub-kernel system: multiple isolated full OS instances inside one kernel.

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
pub mod subkernel;

pub use priv_model::{Ring, RingLevel, Capability, CapabilityToken};
pub use subkernel::{SubKernel, SubKernelConfig, SubKernelId, SubKernelManager};
pub use subkernel::instance::{SubKernelProfile, SubKernelState};
pub use subkernel::bridge::{Bridge, BridgeKind};
