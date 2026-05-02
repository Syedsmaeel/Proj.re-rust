//! Signal-Sovereign IPC
//!
//! Asynchronous signaling between the Substrate and Sub-Kernels.

pub enum Signal {
    Suspend,
    Resume,
    Terminate,
    CapabilityViolation,
}

pub trait SignalHandler {
    fn handle_signal(&mut self, signal: Signal);
}
