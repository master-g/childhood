//! # NES Compiler
//!
//! A modern NES assembler and compiler written in Rust, providing a safe and efficient
//! way to compile 6502 assembly code for the Nintendo Entertainment System.
//!
//! ## Features
//!
//! - Complete 6502 instruction set support
//! - NES-specific directives and functionality
//! - Macro system with parameter substitution
//! - Multi-pass assembly for forward references
//! - Symbol table management with scoping
//! - iNES ROM format output
//! - Comprehensive error reporting
//! - Memory-safe implementation
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use nes_compiler::{Assembler, Config};
//! use std::path::Path;
//!
//! let config = Config::default();
//! let mut assembler = Assembler::new(config);
//!
//! match assembler.assemble_file(Path::new("game.asm")) {
//!     Ok(rom_data) => {
//!         println!("Assembly successful! ROM size: {} bytes", rom_data.len());
//!     }
//!     Err(e) => {
//!         eprintln!("Assembly failed: {}", e);
//!     }
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![cfg_attr(docsrs, feature(doc_cfg))]

// Core modules
pub mod config;
pub mod core;
pub mod error;

// Component modules
pub mod instructions;
pub mod macros;
pub mod output;
pub mod parsing;
pub mod platform;
pub mod symbols;

// Utility modules
pub mod utils;

// Re-exports for convenience
pub use crate::config::Config;
pub use crate::core::assembler::Assembler;
pub use crate::error::{AssemblyError, AssemblyResult};

// Re-export commonly used types
pub use crate::core::memory::{Bank, MemoryManager, Section};
pub use crate::instructions::{AddressingMode, Instruction, Mnemonic};
pub use crate::platform::nes::NesConfig;
pub use crate::symbols::{Symbol, SymbolTable, SymbolType};

/// Library version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default bank size for NES (8KB)
pub const DEFAULT_BANK_SIZE: usize = 8192;

/// Maximum number of banks supported
pub const MAX_BANKS: usize = 4096;

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_version_string() {
		assert!(!VERSION.is_empty());
	}

	#[test]
	fn test_constants() {
		assert_eq!(DEFAULT_BANK_SIZE, 8192);
		assert_eq!(MAX_BANKS, 4096);
	}
}
