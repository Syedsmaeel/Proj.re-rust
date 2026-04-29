//! Timux Sub-Kernel System
//!
//! Each sub-kernel is a complete, isolated OS instance with:
//! - Its own 5-ring privilege model
//! - Its own scheduler
//! - Its own memory manager
//! - Its own driver registry
//! - Its own filesystem interface
//! - Its own capability table
//!
//! Sub-kernels are:
//! - Spawnable by master kernel (ring 0) or other sub-kernels (with SPAWN cap)
//! - Hot-swappable while running
//! - Pausable, cloneable, migratable
//! - Connected via IPC channels and/or shared memory regions

pub mod instance;
pub mod manager;
pub mod bridge;
pub mod snapshot;

pub use instance::{SubKernel, SubKernelId, SubKernelState, SubKernelConfig};
pub use manager::SubKernelManager;
pub use bridge::{Bridge, BridgeKind};
pub use snapshot::SubKernelSnapshot;
