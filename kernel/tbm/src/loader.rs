//! Timux ELF Loader
//!
//! Parses and loads the master kernel binary into memory.

use xmas_elf::{ElfFile, program};
use log::info;
use crate::protocol::BootInfo;

pub struct KernelLoader;

impl KernelLoader {
    /// Loads an ELF binary into memory at the specified physical address.
    pub fn load(elf_data: &'static [u8], _boot_info: &mut BootInfo) -> Result<u64, &'static str> {
        let elf = ElfFile::new(elf_data).map_err(|_| "Failed to parse ELF")?;
        
        // Ensure it's a 64-bit kernel
        assert_eq!(elf.header.pt2.machine().as_u16(), 62, "Not x86_64");

        for program_header in elf.program_iter().map_err(|_| "Invalid program header")? {
            if program_header.get_type().map_err(|_| "Invalid header type")? == program::Type::Load {
                let load_addr = program_header.virtual_addr();
                let mem_size = program_header.mem_size();
                let file_size = program_header.file_size();
                let file_offset = program_header.offset() as usize;

                info!("󰒋 Loading segment at {:#x}", load_addr);

                // Copy segment data to target memory
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        elf_data.as_ptr().add(file_offset),
                        load_addr as *mut u8,
                        file_size as usize,
                    );
                    
                    // Zero out the remaining part of the memory segment
                    if mem_size > file_size {
                        core::ptr::write_bytes(
                            (load_addr + file_size) as *mut u8,
                            0,
                            (mem_size - file_size) as usize,
                        );
                    }
                }
            }
        }

        // Return the entry point
        Ok(elf.header.pt2.entry_point())
    }
}
