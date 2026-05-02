//! Timux — Sovereign Kernel
#![no_std]
#![allow(dead_code)]
extern crate alloc;

pub mod arch;
pub mod bashpp;
pub mod boot;
pub mod cap;
pub mod ipc;
pub mod mm;
pub mod morph;
pub mod storage;
pub mod priv_model;
pub mod sched;
pub mod subkernel;

pub use priv_model::{Ring, RingLevel, Capability, CapabilityToken};
pub use subkernel::{SubKernel, SubKernelConfig, SubKernelId, SubKernelManager};
pub use subkernel::instance::{SubKernelProfile, SubKernelState};
pub use subkernel::bridge::{Bridge, BridgeKind};
pub use mm::allocator::LinkedListAllocator;
pub use bashpp::{Lexer, Token, TokenKind, Parser, Ast, Command, Runtime, Env, TuiShell, Tab, Pane, Widget};
