//! Timux IPC — capability-gated message passing between rings

use crate::priv_model::{CapRight, CapabilityToken, PrivError};

/// An IPC message
#[derive(Debug, Clone)]
pub struct Message {
    pub from:    u64,            // sender task ID
    pub to:      u64,            // target task ID
    pub payload: [u8; 256],     // inline payload
    pub len:     usize,
    pub caps:    alloc::vec::Vec<CapabilityToken>, // attached capabilities
}

impl Message {
    pub fn new(from: u64, to: u64, data: &[u8]) -> Self {
        let mut payload = [0u8; 256];
        let len = data.len().min(256);
        payload[..len].copy_from_slice(&data[..len]);
        Self { from, to, payload, len, caps: alloc::vec::Vec::new() }
    }

    pub fn attach_cap(&mut self, cap: CapabilityToken) {
        self.caps.push(cap);
    }

    pub fn data(&self) -> &[u8] { &self.payload[..self.len] }
}

/// A capability-gated IPC channel
pub struct Channel {
    pub id:    u64,
    queue:     alloc::collections::VecDeque<Message>,
    capacity:  usize,
}

impl Channel {
    pub fn new(id: u64, capacity: usize) -> Self {
        Self { id, queue: alloc::collections::VecDeque::new(), capacity }
    }

    /// Send a message — requires SEND capability
    pub fn send(
        &mut self,
        cap: &CapabilityToken,
        msg: Message,
    ) -> Result<(), PrivError> {
        if !cap.permits(CapRight::SEND) {
            return Err(PrivError::InsufficientRights);
        }
        if self.queue.len() >= self.capacity {
            return Err(PrivError::InsufficientRights); // channel full
        }
        self.queue.push_back(msg);
        Ok(())
    }

    /// Receive a message — requires RECV capability
    pub fn recv(
        &mut self,
        cap: &CapabilityToken,
    ) -> Result<Message, PrivError> {
        if !cap.permits(CapRight::RECV) {
            return Err(PrivError::InsufficientRights);
        }
        self.queue.pop_front().ok_or(PrivError::InsufficientRights)
    }

    pub fn pending(&self) -> usize { self.queue.len() }
}

extern crate alloc;
