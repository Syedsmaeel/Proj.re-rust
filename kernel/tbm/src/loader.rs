//! Timux ELF Loader
//!
//! Parses and loads the master kernel binary into memory using the Sovereign ELF-Gate (SEG).

use log::info;
use crate::protocol::BootInfo;
use re_core::seg::ElfParser;

pub struct KernelLoader;

impl KernelLoader {
    /// Loads an ELF binary into memory using the SEG parser.
    pub fn load(elf_data: &'static [u8], _boot_info: &mut BootInfo) -> Result<u64, &'static str> {
        if !ElfParser::is_valid(elf_data) {
            return Err("Invalid ELF binary");
        }

        let header = ElfParser::header(elf_data);
        let phnum = header.phnum;
        let entry = header.entry;
        
        info!("󰒋 Loading Sovereign ELF binary, segments: {}", phnum);

        for i in 0..phnum {
            if let Some(ph) = ElfParser::program_header(elf_data, i) {
                // Program Type 1 is LOAD
                if ph.type_ == 1 {
                    let load_addr = ph.vaddr as *mut u8;
                    let file_size = ph.filesz as usize;
                    let mem_size = ph.memsz as usize;
                    let offset = ph.offset as usize;
                    let ph_offset = ph.offset;
                    let ph_vaddr = ph.vaddr;

                    info!("󰒋 Mapping segment: {:#x} -> {:#x}", ph_offset, ph_vaddr);

                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            elf_data.as_ptr().add(offset),
                            load_addr,
                            file_size,
                        );
                        
                        if mem_size > file_size {
                            core::ptr::write_bytes(
                                load_addr.add(file_size),
                                0,
                                mem_size - file_size,
                            );
                        }
                    }
                }
            }
        }

        Ok(entry)
    }
}
