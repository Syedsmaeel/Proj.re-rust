//! Sub-Kernel Driver Registry
//! 
//! Allows sub-kernels to register and own their own device drivers.

use alloc::vec::Vec;
use crate::priv_model::CapabilityToken;

pub struct Driver {
    pub name: &'static str,
    pub cap: CapabilityToken,
}

pub struct DriverRegistry {
    pub drivers: Vec<Driver>,
}

impl DriverRegistry {
    pub fn new() -> Self {
        Self { drivers: Vec::new() }
    }

    pub fn register(&mut self, name: &'static str, cap: CapabilityToken) {
        self.drivers.push(Driver { name, cap });
    }
}
