//! Output generation for the NES compiler
//!
//! This module handles the generation of various output formats including
//! ROM files, listing files, and symbol export files.

pub mod listing;
pub mod rom;
pub mod symbols;

pub use listing::{ListingGenerator, ListingLevel};
pub use rom::{RomFormat, RomGenerator};
pub use symbols::{SymbolExporter, SymbolFormat};

use crate::error::AssemblyResult;
use std::path::Path;

/// Main output manager that coordinates all output generation
#[derive(Debug)]
pub struct OutputManager {
	/// ROM generator
	rom_generator: RomGenerator,
	/// Listing generator
	listing_generator: ListingGenerator,
	/// Symbol exporter
	symbol_exporter: SymbolExporter,
}

impl OutputManager {
	/// Create a new output manager
	pub fn new() -> Self {
		Self {
			rom_generator: RomGenerator::new(),
			listing_generator: ListingGenerator::new(),
			symbol_exporter: SymbolExporter::new(),
		}
	}

	/// Generate ROM file
	pub fn generate_rom(&self, data: &[u8], output_path: &Path) -> AssemblyResult<()> {
		self.rom_generator.generate(data, output_path)
	}

	/// Generate listing file
	pub fn generate_listing(&self, output_path: &Path) -> AssemblyResult<()> {
		self.listing_generator.generate(output_path)
	}

	/// Generate symbol files
	pub fn generate_symbols(&self, output_path: &Path) -> AssemblyResult<()> {
		self.symbol_exporter.generate(output_path)
	}
}

impl Default for OutputManager {
	fn default() -> Self {
		Self::new()
	}
}
