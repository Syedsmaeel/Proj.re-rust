//! Sovereign Onion-Routing Protocol (SOP)
//!
//! A capability-gated, fractal-native tunnel protocol.

#[repr(C, packed)]
pub struct OnionFrame {
    pub next_hop: u64,           // Sub-kernel ID of next relay
    pub payload_len: usize,
    pub signature: [u8; 32],     // Capability token signature
    pub data: [u8; 512],         // Encapsulated payload
}

pub trait OnionRelay {
    fn can_relay(&self) -> bool;
    fn relay(&mut self, frame: OnionFrame) -> Result<(), &'static str>;
}
