//! RISC-V ELF binary loader.
//!
//! Parses ELF32 binaries and loads them into memory for execution.

use crate::error::ExecutorError;
use crate::memory::Memory;

/// ELF magic number.
const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];

/// ELF class: 32-bit.
const ELFCLASS32: u8 = 1;

/// ELF data encoding: little-endian.
const ELFDATA2LSB: u8 = 1;

/// ELF machine type: RISC-V.
const EM_RISCV: u16 = 243;

/// Program header type: loadable segment.
const PT_LOAD: u32 = 1;

/// ELF file header (32-bit).
#[derive(Debug, Clone)]
pub struct Elf32Header {
    /// Entry point address.
    pub entry: u32,
    /// Program header table offset.
    pub phoff: u32,
    /// Section header table offset.
    pub shoff: u32,
    /// Processor-specific flags.
    pub flags: u32,
    /// ELF header size.
    pub ehsize: u16,
    /// Program header entry size.
    pub phentsize: u16,
    /// Number of program header entries.
    pub phnum: u16,
    /// Section header entry size.
    pub shentsize: u16,
    /// Number of section header entries.
    pub shnum: u16,
    /// Section name string table index.
    pub shstrndx: u16,
}

/// Program header (32-bit).
#[derive(Debug, Clone)]
pub struct Elf32ProgramHeader {
    /// Segment type.
    pub p_type: u32,
    /// Offset in file.
    pub p_offset: u32,
    /// Virtual address.
    pub p_vaddr: u32,
    /// Physical address.
    pub p_paddr: u32,
    /// Size in file.
    pub p_filesz: u32,
    /// Size in memory.
    pub p_memsz: u32,
    /// Segment flags.
    pub p_flags: u32,
    /// Alignment.
    pub p_align: u32,
}

/// ELF loader for RISC-V binaries.
pub struct ElfLoader {
    /// Raw ELF data.
    data: Vec<u8>,
    /// Parsed header.
    header: Elf32Header,
    /// Parsed program headers.
    program_headers: Vec<Elf32ProgramHeader>,
}

impl ElfLoader {
    /// Parse an ELF file from bytes.
    pub fn parse(data: &[u8]) -> Result<Self, ExecutorError> {
        if data.len() < 52 {
            return Err(ExecutorError::InvalidElf("File too small".into()));
        }

        // Check magic number
        if data[0..4] != ELF_MAGIC {
            return Err(ExecutorError::InvalidElf("Invalid ELF magic".into()));
        }

        // Check class (32-bit)
        if data[4] != ELFCLASS32 {
            return Err(ExecutorError::InvalidElf("Not a 32-bit ELF".into()));
        }

        // Check endianness (little-endian)
        if data[5] != ELFDATA2LSB {
            return Err(ExecutorError::InvalidElf("Not little-endian".into()));
        }

        // Check machine type (RISC-V)
        let machine = u16::from_le_bytes([data[18], data[19]]);
        if machine != EM_RISCV {
            return Err(ExecutorError::InvalidElf(
                format!("Not a RISC-V ELF (machine type: {})", machine)
            ));
        }

        // Parse header
        let header = Elf32Header {
            entry: u32::from_le_bytes([data[24], data[25], data[26], data[27]]),
            phoff: u32::from_le_bytes([data[28], data[29], data[30], data[31]]),
            shoff: u32::from_le_bytes([data[32], data[33], data[34], data[35]]),
            flags: u32::from_le_bytes([data[36], data[37], data[38], data[39]]),
            ehsize: u16::from_le_bytes([data[40], data[41]]),
            phentsize: u16::from_le_bytes([data[42], data[43]]),
            phnum: u16::from_le_bytes([data[44], data[45]]),
            shentsize: u16::from_le_bytes([data[46], data[47]]),
            shnum: u16::from_le_bytes([data[48], data[49]]),
            shstrndx: u16::from_le_bytes([data[50], data[51]]),
        };

        // Parse program headers
        let mut program_headers = Vec::new();
        let phoff = header.phoff as usize;
        let phentsize = header.phentsize as usize;

        for i in 0..header.phnum as usize {
            let offset = phoff + i * phentsize;
            if offset + 32 > data.len() {
                return Err(ExecutorError::InvalidElf("Program header out of bounds".into()));
            }

            let ph = Elf32ProgramHeader {
                p_type: u32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]]),
                p_offset: u32::from_le_bytes([data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7]]),
                p_vaddr: u32::from_le_bytes([data[offset + 8], data[offset + 9], data[offset + 10], data[offset + 11]]),
                p_paddr: u32::from_le_bytes([data[offset + 12], data[offset + 13], data[offset + 14], data[offset + 15]]),
                p_filesz: u32::from_le_bytes([data[offset + 16], data[offset + 17], data[offset + 18], data[offset + 19]]),
                p_memsz: u32::from_le_bytes([data[offset + 20], data[offset + 21], data[offset + 22], data[offset + 23]]),
                p_flags: u32::from_le_bytes([data[offset + 24], data[offset + 25], data[offset + 26], data[offset + 27]]),
                p_align: u32::from_le_bytes([data[offset + 28], data[offset + 29], data[offset + 30], data[offset + 31]]),
            };

            program_headers.push(ph);
        }

        Ok(Self {
            data: data.to_vec(),
            header,
            program_headers,
        })
    }

    /// Get the entry point address.
    pub fn entry_point(&self) -> u32 {
        self.header.entry
    }

    /// Get loadable segments.
    pub fn loadable_segments(&self) -> impl Iterator<Item = &Elf32ProgramHeader> {
        self.program_headers.iter().filter(|ph| ph.p_type == PT_LOAD)
    }

    /// Load the ELF into memory.
    pub fn load_into_memory(&self, memory: &mut Memory) -> Result<u32, ExecutorError> {
        for ph in self.loadable_segments() {
            let file_offset = ph.p_offset as usize;
            let file_size = ph.p_filesz as usize;
            let mem_addr = ph.p_vaddr;
            let mem_size = ph.p_memsz as usize;

            // Validate bounds
            if file_offset + file_size > self.data.len() {
                return Err(ExecutorError::InvalidElf("Segment data out of bounds".into()));
            }

            // Load file data into memory
            if file_size > 0 {
                let segment_data = &self.data[file_offset..file_offset + file_size];
                memory.load_program(mem_addr, segment_data)?;
            }

            // Zero-fill BSS section (memsz > filesz)
            if mem_size > file_size {
                let bss_start = mem_addr + file_size as u32;
                let bss_size = mem_size - file_size;
                for i in 0..bss_size {
                    memory.write_u8(bss_start + i as u32, 0)?;
                }
            }
        }

        Ok(self.entry_point())
    }

    /// Get memory requirements (lowest and highest addresses).
    pub fn memory_bounds(&self) -> (u32, u32) {
        let mut low = u32::MAX;
        let mut high = 0u32;

        for ph in self.loadable_segments() {
            low = low.min(ph.p_vaddr);
            high = high.max(ph.p_vaddr + ph.p_memsz);
        }

        (low, high)
    }

    /// Get total memory size needed.
    pub fn total_memory_size(&self) -> u32 {
        let (low, high) = self.memory_bounds();
        if high > low {
            high - low
        } else {
            0
        }
    }

    /// Get ELF header.
    pub fn header(&self) -> &Elf32Header {
        &self.header
    }

    /// Get program headers.
    pub fn program_headers(&self) -> &[Elf32ProgramHeader] {
        &self.program_headers
    }
}

/// ELF section flags.
pub mod section_flags {
    /// Section is writable.
    pub const SHF_WRITE: u32 = 0x1;
    /// Section occupies memory during execution.
    pub const SHF_ALLOC: u32 = 0x2;
    /// Section is executable.
    pub const SHF_EXECINSTR: u32 = 0x4;
}

/// ELF segment flags.
pub mod segment_flags {
    /// Segment is executable.
    pub const PF_X: u32 = 0x1;
    /// Segment is writable.
    pub const PF_W: u32 = 0x2;
    /// Segment is readable.
    pub const PF_R: u32 = 0x4;
}

/// Build a minimal ELF header for testing.
pub fn build_test_elf(code: &[u8], entry: u32, load_addr: u32) -> Vec<u8> {
    let mut elf = Vec::new();
    
    // ELF header (52 bytes)
    elf.extend_from_slice(&ELF_MAGIC);
    elf.push(ELFCLASS32);           // Class: 32-bit
    elf.push(ELFDATA2LSB);          // Data: little-endian
    elf.push(1);                    // Version
    elf.push(0);                    // OS/ABI
    elf.extend_from_slice(&[0u8; 8]); // Padding
    elf.extend_from_slice(&2u16.to_le_bytes()); // Type: executable
    elf.extend_from_slice(&EM_RISCV.to_le_bytes()); // Machine: RISC-V
    elf.extend_from_slice(&1u32.to_le_bytes()); // Version
    elf.extend_from_slice(&entry.to_le_bytes()); // Entry point
    elf.extend_from_slice(&52u32.to_le_bytes()); // Program header offset
    elf.extend_from_slice(&0u32.to_le_bytes()); // Section header offset
    elf.extend_from_slice(&0u32.to_le_bytes()); // Flags
    elf.extend_from_slice(&52u16.to_le_bytes()); // ELF header size
    elf.extend_from_slice(&32u16.to_le_bytes()); // Program header entry size
    elf.extend_from_slice(&1u16.to_le_bytes()); // Number of program headers
    elf.extend_from_slice(&0u16.to_le_bytes()); // Section header entry size
    elf.extend_from_slice(&0u16.to_le_bytes()); // Number of section headers
    elf.extend_from_slice(&0u16.to_le_bytes()); // Section name string table index
    
    // Program header (32 bytes)
    let code_offset = 52 + 32; // After ELF header + program header
    elf.extend_from_slice(&PT_LOAD.to_le_bytes()); // Type: loadable
    elf.extend_from_slice(&(code_offset as u32).to_le_bytes()); // Offset
    elf.extend_from_slice(&load_addr.to_le_bytes()); // Virtual address
    elf.extend_from_slice(&load_addr.to_le_bytes()); // Physical address
    elf.extend_from_slice(&(code.len() as u32).to_le_bytes()); // File size
    elf.extend_from_slice(&(code.len() as u32).to_le_bytes()); // Memory size
    elf.extend_from_slice(&(segment_flags::PF_R | segment_flags::PF_X).to_le_bytes()); // Flags
    elf.extend_from_slice(&4u32.to_le_bytes()); // Alignment
    
    // Code segment
    elf.extend_from_slice(code);
    
    elf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_and_parse_elf() {
        // Simple RISC-V code: addi x1, x0, 42; ecall
        let code = vec![
            0x93, 0x00, 0xa0, 0x02, // addi x1, x0, 42
            0x73, 0x00, 0x00, 0x00, // ecall
        ];
        
        let elf_data = build_test_elf(&code, 0x1000, 0x1000);
        let loader = ElfLoader::parse(&elf_data).expect("Failed to parse ELF");
        
        assert_eq!(loader.entry_point(), 0x1000);
        assert_eq!(loader.loadable_segments().count(), 1);
    }

    #[test]
    fn test_load_into_memory() {
        let code = vec![
            0x93, 0x00, 0xa0, 0x02, // addi x1, x0, 42
            0x73, 0x00, 0x00, 0x00, // ecall
        ];
        
        let elf_data = build_test_elf(&code, 0x1000, 0x1000);
        let loader = ElfLoader::parse(&elf_data).unwrap();
        
        let mut memory = Memory::with_default_size();
        let entry = loader.load_into_memory(&mut memory).unwrap();
        
        assert_eq!(entry, 0x1000);
        
        // Verify code was loaded
        let instr = memory.read_u32(0x1000).unwrap();
        assert_eq!(instr, 0x02a00093); // addi x1, x0, 42
    }

    #[test]
    fn test_invalid_elf() {
        let bad_data = vec![0x00, 0x01, 0x02, 0x03];
        let result = ElfLoader::parse(&bad_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_memory_bounds() {
        let code = vec![0x00; 100];
        let elf_data = build_test_elf(&code, 0x2000, 0x2000);
        let loader = ElfLoader::parse(&elf_data).unwrap();
        
        let (low, high) = loader.memory_bounds();
        assert_eq!(low, 0x2000);
        assert_eq!(high, 0x2000 + 100);
    }
}
