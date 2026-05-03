//! Timux Boot Protocol (TBP)
//!
//! Shared definitions for system handoff.

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubKernelConfig {
    pub name: StaticStr,
    pub os_type: StaticStr,
    pub rings: u8,
    pub capabilities: [u64; 4], // Bitflags for simplicity
    pub personality_script: StaticStr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blueprint {
    pub system_name: StaticStr,
    pub substrate_rings: u8,
    pub subkernels: [Option<SubKernelConfig>; 16],
    pub origin_entropy: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct StaticStr {
    ptr: *const u8,
    len: usize,
}

impl StaticStr {
    pub const fn new(s: &'static str) -> Self {
        Self {
            ptr: s.as_ptr(),
            len: s.len(),
        }
    }

    pub fn as_str(&self) -> &'static str {
        unsafe {
            let slice = core::slice::from_raw_parts(self.ptr, self.len);
            core::str::from_utf8_unchecked(slice)
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct BootInfo {
    pub magic: u64,             // 0x54494d55582d3121 ('TIMUX-1!')
    pub version: u32,
    
    // Memory and Hardware
    pub heap_start: usize,
    pub heap_size: usize,
    pub framebuffer: FramebufferInfo,
    pub mmap_addr: usize,
    pub mmap_len: usize,
    
    // Sovereign Configurations (The injected configs)
    pub blueprint_addr: usize,  // Pointer to the serialized Sovereign Blueprint
    pub blueprint_len: usize,
    
    // Entropy for early security
    pub entropy_seed: [u8; 32],
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FramebufferInfo {
    pub addr: u64,
    pub size: usize,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
}


extern crate alloc;
use alloc::boxed::Box;
use alloc::string::String;

impl Serialize for StaticStr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for StaticStr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct StrVisitor;

        impl<'de> serde::de::Visitor<'de> for StrVisitor {
            type Value = &'static str;

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where E: serde::de::Error,
            {
                let boxed = Box::<str>::from(value);
                Ok(Box::leak(boxed))
            }

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("a string")
            }
        }

        deserializer.deserialize_str(StrVisitor).map(StaticStr::new)
    }
}
