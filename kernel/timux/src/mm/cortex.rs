//! Memory-Cortex Fabric
//!
//! Defines the semantic memory context objects used by the sovereign substrate.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Volatility {
    Ghost,      // Volatile, zero-commit, panic-erased
    Persistent, // Committed to vDisk snapshot
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryContext {
    pub owner_sk: u64,           // SubKernelId
    pub semantic_tag: [u8; 16],  // e.g., "NetworkStack.Buf"
    pub volatility: Volatility,
    pub security_zone: u8,
}

impl MemoryContext {
    pub fn new(owner: u64, tag: &str, volatility: Volatility) -> Self {
        let mut semantic_tag = [0u8; 16];
        let bytes = tag.as_bytes();
        let len = bytes.len().min(16);
        semantic_tag[..len].copy_from_slice(&bytes[..len]);
        
        Self {
            owner_sk: owner,
            semantic_tag,
            volatility,
            security_zone: 0,
        }
    }
}
