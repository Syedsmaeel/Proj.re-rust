//! Sovereign ELF-Gate (SEG)
//!
//! A minimal, secure, and capability-aware ELF parser 
//! built specifically for the Timux Sovereign Substrate.

#[repr(C, packed)]
pub struct ElfHeader {
    pub magic: [u8; 4],
    pub class: u8,
    pub data: u8,
    pub version: u8,
    pub os_abi: u8,
    pub abi_version: u8,
    pub pad: [u8; 7],
    pub type_: u16,
    pub machine: u16,
    pub version2: u32,
    pub entry: u64,
    pub phoff: u64,
    pub shoff: u64,
    pub flags: u32,
    pub ehsize: u16,
    pub phentsize: u16,
    pub phnum: u16,
    pub shentsize: u16,
    pub shnum: u16,
    pub shstrndx: u16,
}

#[repr(C, packed)]
pub struct ProgramHeader {
    pub type_: u32,
    pub flags: u32,
    pub offset: u64,
    pub vaddr: u64,
    pub paddr: u64,
    pub filesz: u64,
    pub memsz: u64,
    pub align: u64,
}

pub struct ElfParser;

impl ElfParser {
    pub fn is_valid(data: &[u8]) -> bool {
        data.len() >= 64 && &data[0..4] == b"\x7fELF"
    }

    pub fn header(data: &[u8]) -> &ElfHeader {
        unsafe { &*(data.as_ptr() as *const ElfHeader) }
    }

    pub fn program_header(data: &[u8], index: u16) -> Option<&ProgramHeader> {
        let header = Self::header(data);
        if index >= header.phnum { return None; }
        
        let offset = header.phoff as usize + (index as usize * header.phentsize as usize);
        if offset + header.phentsize as usize > data.len() { return None; }
        
        unsafe { Some(&*(data.as_ptr().add(offset) as *const ProgramHeader)) }
    }
}
