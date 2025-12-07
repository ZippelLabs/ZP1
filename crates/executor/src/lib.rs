//! zp1-executor: Deterministic RISC-V RV32IM executor with trace emission.
//!
//! This crate provides:
//! - A minimal RV32IM CPU emulator (no MMU, no privileged modes)
//! - Execution trace output for proving
//! - ELF binary loader
//! - Syscall/precompile hooks for delegation

pub mod cpu;
pub mod memory;
pub mod decode;
pub mod trace;
pub mod error;
pub mod elf;
pub mod syscall;

pub use cpu::Cpu;
pub use memory::Memory;
pub use trace::{ExecutionTrace, TraceRow};
pub use error::ExecutorError;
pub use elf::ElfLoader;
pub use syscall::SyscallCode;
