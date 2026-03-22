use std::fs;
use std::path::Path;

use goblin::elf::Elf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ElfError {
    #[error("Failed to read file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse ELF: {0}")]
    Parse(#[from] goblin::error::Error),
    #[error("Not a RISC-V ELF binary")]
    NotRiscV,
    #[error("No code segment found")]
    NoCode,
}

#[derive(Debug)]
pub struct LoadedBinary {
    pub name: String,
    pub entry_point: u64,
    pub code: Vec<u8>,
    pub code_start: u64,
    pub code_size: u64,
    pub data: Vec<u8>,
    pub data_start: u64,
    pub data_size: u64,
    pub is_64bit: bool,
}

impl LoadedBinary {
    pub fn code_end(&self) -> u64 {
        self.code_start + self.code_size
    }

    pub fn data_end(&self) -> u64 {
        self.data_start + self.data_size
    }
}

pub fn load_elf(path: &Path) -> Result<LoadedBinary, ElfError> {
    let bytes = fs::read(path)?;
    let elf = Elf::parse(&bytes)?;

    // Check it's RISC-V
    if elf.header.e_machine != goblin::elf::header::EM_RISCV {
        return Err(ElfError::NotRiscV);
    }

    let is_64bit = elf.is_64;
    let entry_point = elf.entry;

    let mut code = Vec::new();
    let mut code_start = 0u64;
    let mut data = Vec::new();
    let mut data_start = 0u64;

    // Extract loadable segments
    for ph in &elf.program_headers {
        if ph.p_type == goblin::elf::program_header::PT_LOAD {
            let offset = ph.p_offset as usize;
            let filesz = ph.p_filesz as usize;
            let memsz = ph.p_memsz as usize;
            let vaddr = ph.p_vaddr;

            let mut segment_data = Vec::with_capacity(memsz);
            if offset + filesz <= bytes.len() {
                segment_data.extend_from_slice(&bytes[offset..offset + filesz]);
            }
            // Zero-fill remaining (BSS)
            segment_data.resize(memsz, 0);

            if ph.p_flags & goblin::elf::program_header::PF_X != 0 {
                // Executable segment = code
                code_start = vaddr;
                code = segment_data;
            } else {
                // Data segment
                if data.is_empty() {
                    data_start = vaddr;
                    data = segment_data;
                } else {
                    // Append to existing data (handle gaps)
                    let current_end = data_start + data.len() as u64;
                    if vaddr > current_end {
                        let gap = (vaddr - current_end) as usize;
                        data.resize(data.len() + gap, 0);
                    }
                    data.extend_from_slice(&segment_data);
                }
            }
        }
    }

    if code.is_empty() {
        return Err(ElfError::NoCode);
    }

    let code_size = code.len() as u64;
    let data_size = data.len() as u64;

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    Ok(LoadedBinary {
        name,
        entry_point,
        code,
        code_start,
        code_size,
        data,
        data_start,
        data_size,
        is_64bit,
    })
}
