//! Timux — Sovereign Kernel
  #![no_std]
  extern crate alloc;

  pub mod arch;
  pub mod bashpp;
  pub mod boot;
  pub mod cap;
  pub mod ikfs;
  pub mod ipc;
  pub mod mm;
  pub mod morph;
  pub mod storage;
  pub mod priv_model;
  pub mod sched;
  pub mod subkernel;

  pub use priv_model::{Ring, RingLevel, Capability, CapabilityToken};
  pub use subkernel::{SubKernel, SubKernelConfig, SubKernelId, SubKernelManager};
  pub use subkernel::{SubKernelProfile, SubKernelState, MemoryRange};
  pub use subkernel::{ShadowManager, ShadowInstance};
  pub use mm::allocator::LinkedListAllocator;
  pub use bashpp::{Lexer, Token, TokenKind, Parser, Ast, Command, Runtime, Env, TuiShell, Tab, Pane, Widget};
  pub use ikfs::Ikfs;
  