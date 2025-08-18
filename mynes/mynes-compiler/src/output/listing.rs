//! Listing file generation for the NES assembler.
//!
//! This module provides functionality to generate detailed assembly listing files
//! that show the original source code alongside the generated machine code.

use crate::core::memory::MemoryManager;
use crate::error::{AssemblyError, AssemblyResult, SourcePos};
use crate::parsing::Statement;
use crate::symbols::SymbolManager;
use std::collections::HashMap;
use std::fmt::Write;
use std::fs::File;
use std::io::{BufWriter, Write as IoWrite};
use std::path::{Path, PathBuf};

/// Listing output levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListingLevel {
	/// No listing output
	None = 0,
	/// Basic listing with addresses and opcodes
	Basic = 1,
	/// Standard listing with source and symbols
	Standard = 2,
	/// Detailed listing with cross-references
	Detailed = 3,
}

impl Default for ListingLevel {
	fn default() -> Self {
		Self::Standard
	}
}

impl From<u8> for ListingLevel {
	fn from(value: u8) -> Self {
		match value {
			0 => Self::None,
			1 => Self::Basic,
			2 => Self::Standard,
			3 => Self::Detailed,
			_ => Self::Standard,
		}
	}
}

/// A single line in the listing output
#[derive(Debug, Clone)]
pub struct ListingLine {
	/// Line number in source file
	pub line_number: usize,
	/// Memory address (if applicable)
	pub address: Option<u16>,
	/// Generated machine code bytes
	pub bytes: Vec<u8>,
	/// Original source line
	pub source: String,
	/// File path
	pub file: PathBuf,
	/// Whether this line has an error
	pub has_error: bool,
	/// Error message (if any)
	pub error_message: Option<String>,
	/// Whether to show macro expansion
	pub is_macro_expansion: bool,
	/// Nesting level for macro expansions
	pub macro_level: usize,
}

impl ListingLine {
	/// Create a new listing line
	pub fn new(
		line_number: usize,
		address: Option<u16>,
		bytes: Vec<u8>,
		source: String,
		file: PathBuf,
	) -> Self {
		Self {
			line_number,
			address,
			bytes,
			source,
			file,
			has_error: false,
			error_message: None,
			is_macro_expansion: false,
			macro_level: 0,
		}
	}

	/// Mark line as having an error
	pub fn with_error(mut self, message: String) -> Self {
		self.has_error = true;
		self.error_message = Some(message);
		self
	}

	/// Mark line as macro expansion
	pub fn with_macro_expansion(mut self, level: usize) -> Self {
		self.is_macro_expansion = true;
		self.macro_level = level;
		self
	}

	/// Format the line for output
	pub fn format(&self, level: ListingLevel, show_file: bool) -> String {
		let mut output = String::new();

		match level {
			ListingLevel::None => return String::new(),
			ListingLevel::Basic => {
				// Line: ADDR BYTES
				if let Some(addr) = self.address {
					write!(output, "{:04X} ", addr).unwrap();
				} else {
					output.push_str("     ");
				}

				// Format bytes (up to 3 bytes per line)
				for (i, byte) in self.bytes.iter().take(3).enumerate() {
					if i > 0 {
						output.push(' ');
					}
					write!(output, "{:02X}", byte).unwrap();
				}

				// Pad to consistent width
				let bytes_width = self.bytes.len().min(3) * 2 + (self.bytes.len().min(3) - 1);
				for _ in bytes_width..8 {
					output.push(' ');
				}
			}
			ListingLevel::Standard | ListingLevel::Detailed => {
				// Line number
				write!(output, "{:4} ", self.line_number).unwrap();

				// Address
				if let Some(addr) = self.address {
					write!(output, "{:04X} ", addr).unwrap();
				} else {
					output.push_str("     ");
				}

				// Bytes (up to 3 bytes, then continuation lines)
				for (i, byte) in self.bytes.iter().take(3).enumerate() {
					if i > 0 {
						output.push(' ');
					}
					write!(output, "{:02X}", byte).unwrap();
				}

				// Pad bytes field
				let bytes_shown = self.bytes.len().min(3);
				let bytes_width = bytes_shown * 2 + (bytes_shown.saturating_sub(1));
				for _ in bytes_width..8 {
					output.push(' ');
				}

				output.push(' ');

				// Macro indentation
				if self.is_macro_expansion {
					for _ in 0..self.macro_level {
						output.push_str("  ");
					}
					output.push_str("+ ");
				}

				// Source line
				output.push_str(&self.source);

				// File information (if requested and detailed)
				if show_file && level == ListingLevel::Detailed {
					write!(output, " ; {}", self.file.display()).unwrap();
				}
			}
		}

		// Add error message if present
		if let Some(error) = &self.error_message {
			output.push('\n');
			for _ in 0..10 {
				output.push(' ');
			}
			write!(output, "*** ERROR: {}", error).unwrap();
		}

		output
	}

	/// Check if this line should show continuation for additional bytes
	pub fn needs_continuation(&self) -> bool {
		self.bytes.len() > 3
	}

	/// Get continuation lines for additional bytes
	pub fn continuation_lines(&self) -> Vec<String> {
		let mut lines = Vec::new();

		if self.bytes.len() <= 3 {
			return lines;
		}

		for chunk in self.bytes[3..].chunks(3) {
			let mut line = String::new();

			// Line number placeholder
			line.push_str("     ");

			// Address placeholder
			line.push_str("     ");

			// Bytes
			for (i, byte) in chunk.iter().enumerate() {
				if i > 0 {
					line.push(' ');
				}
				write!(line, "{:02X}", byte).unwrap();
			}

			lines.push(line);
		}

		lines
	}
}

/// Configuration for listing generation
#[derive(Debug, Clone)]
pub struct ListingConfig {
	/// Output level
	pub level: ListingLevel,
	/// Show macro expansions
	pub show_macro_expansions: bool,
	/// Show file names in detailed mode
	pub show_file_names: bool,
	/// Maximum width for source lines
	pub max_source_width: usize,
	/// Include symbol cross-references
	pub include_cross_refs: bool,
	/// Include memory usage statistics
	pub include_memory_stats: bool,
	/// Include symbol table
	pub include_symbol_table: bool,
}

impl Default for ListingConfig {
	fn default() -> Self {
		Self {
			level: ListingLevel::Standard,
			show_macro_expansions: false,
			show_file_names: false,
			max_source_width: 80,
			include_cross_refs: false,
			include_memory_stats: true,
			include_symbol_table: true,
		}
	}
}

/// Listing file generator
#[derive(Debug)]
pub struct ListingGenerator {
	/// Configuration
	config: ListingConfig,
	/// Collected listing lines
	lines: Vec<ListingLine>,
	/// Source file contents (for reference)
	source_files: HashMap<PathBuf, Vec<String>>,
	/// Current file being processed
	current_file: Option<PathBuf>,
	/// Current line number
	current_line: usize,
	/// Current address
	current_address: Option<u16>,
	/// Macro expansion level
	macro_level: usize,
}

impl ListingGenerator {
	/// Create a new listing generator
	pub fn new() -> Self {
		Self {
			config: ListingConfig::default(),
			lines: Vec::new(),
			source_files: HashMap::new(),
			current_file: None,
			current_line: 0,
			current_address: None,
			macro_level: 0,
		}
	}

	/// Create with custom configuration
	pub fn with_config(config: ListingConfig) -> Self {
		Self {
			config,
			lines: Vec::new(),
			source_files: HashMap::new(),
			current_file: None,
			current_line: 0,
			current_address: None,
			macro_level: 0,
		}
	}

	/// Add a source file
	pub fn add_source_file(&mut self, file_path: PathBuf, content: &str) {
		let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
		self.source_files.insert(file_path, lines);
	}

	/// Set current file
	pub fn set_current_file(&mut self, file_path: PathBuf) {
		self.current_file = Some(file_path);
		self.current_line = 0;
	}

	/// Set current address
	pub fn set_current_address(&mut self, address: u16) {
		self.current_address = Some(address);
	}

	/// Add a listing line
	pub fn add_line(
		&mut self,
		statement: &Statement,
		bytes: Vec<u8>,
		address: Option<u16>,
	) -> AssemblyResult<()> {
		let pos = statement.pos();
		let file = pos.map(|p| p.file.clone()).or_else(|| self.current_file.clone());
		let line_num = pos.map(|p| p.line).unwrap_or(self.current_line);

		if let Some(file_path) = file {
			// Get source line
			let source_line = self.get_source_line(&file_path, line_num);

			let mut listing_line =
				ListingLine::new(line_num, address, bytes, source_line, file_path);

			// Handle macro expansion
			if self.macro_level > 0 && self.config.show_macro_expansions {
				listing_line = listing_line.with_macro_expansion(self.macro_level);
			}

			self.lines.push(listing_line);
		}

		Ok(())
	}

	/// Add an error line
	pub fn add_error(&mut self, error: &AssemblyError) {
		if let Some(pos) = error.pos() {
			let source_line = self.get_source_line(&pos.file, pos.line);

			let listing_line = ListingLine::new(
				pos.line,
				self.current_address,
				Vec::new(),
				source_line,
				pos.file.clone(),
			)
			.with_error(error.to_string());

			self.lines.push(listing_line);
		}
	}

	/// Enter macro expansion
	pub fn enter_macro(&mut self, _macro_name: &str) {
		self.macro_level += 1;
	}

	/// Exit macro expansion
	pub fn exit_macro(&mut self) {
		if self.macro_level > 0 {
			self.macro_level -= 1;
		}
	}

	/// Generate listing file
	pub fn generate(&self, output_path: &Path) -> AssemblyResult<()> {
		if self.config.level == ListingLevel::None {
			return Ok(());
		}

		let file = File::create(output_path).map_err(|e| AssemblyError::Io {
			pos: Some(SourcePos::file_only(output_path.to_path_buf())),
			source: e,
		})?;

		let mut writer = BufWriter::new(file);
		let content = self.format_listing()?;

		writer.write_all(content.as_bytes()).map_err(|e| AssemblyError::Io {
			pos: Some(SourcePos::file_only(output_path.to_path_buf())),
			source: e,
		})?;

		writer.flush().map_err(|e| AssemblyError::Io {
			pos: Some(SourcePos::file_only(output_path.to_path_buf())),
			source: e,
		})?;

		Ok(())
	}

	/// Generate listing content as string
	pub fn format_listing(&self) -> AssemblyResult<String> {
		let mut output = String::new();

		// Header
		self.write_header(&mut output)?;

		// Main listing
		for line in &self.lines {
			let formatted = line.format(self.config.level, self.config.show_file_names);
			output.push_str(&formatted);
			output.push('\n');

			// Add continuation lines for long instructions
			if line.needs_continuation() {
				for continuation in line.continuation_lines() {
					output.push_str(&continuation);
					output.push('\n');
				}
			}
		}

		// Footer sections
		if self.config.include_symbol_table {
			output.push('\n');
			self.write_symbol_table(&mut output)?;
		}

		if self.config.include_memory_stats {
			output.push('\n');
			self.write_memory_stats(&mut output)?;
		}

		if self.config.include_cross_refs && self.config.level == ListingLevel::Detailed {
			output.push('\n');
			self.write_cross_references(&mut output)?;
		}

		Ok(output)
	}

	/// Write listing header
	fn write_header(&self, output: &mut String) -> AssemblyResult<()> {
		writeln!(output, "NES Assembler Listing").unwrap();
		writeln!(output, "=====================").unwrap();
		writeln!(
			output,
			"Generated: {}",
			std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.map(|d| format!("{} seconds since epoch", d.as_secs()))
				.unwrap_or_else(|_| "unknown time".to_string())
		)
		.unwrap();
		writeln!(output).unwrap();

		// Column headers based on level
		match self.config.level {
			ListingLevel::None => {}
			ListingLevel::Basic => {
				writeln!(output, "ADDR BYTES").unwrap();
				writeln!(output, "---- --------").unwrap();
			}
			ListingLevel::Standard | ListingLevel::Detailed => {
				writeln!(output, "LINE ADDR BYTES    SOURCE").unwrap();
				writeln!(output, "---- ---- -------- ------").unwrap();
			}
		}

		Ok(())
	}

	/// Write symbol table
	fn write_symbol_table(&self, output: &mut String) -> AssemblyResult<()> {
		writeln!(output, "Symbol Table").unwrap();
		writeln!(output, "============").unwrap();
		writeln!(output, "NAME                 VALUE    TYPE").unwrap();
		writeln!(output, "-------------------- -------- --------").unwrap();

		// This would need access to the symbol manager
		// For now, just write a placeholder
		writeln!(output, "(Symbol table would be populated with actual symbols)").unwrap();

		Ok(())
	}

	/// Write memory usage statistics
	fn write_memory_stats(&self, output: &mut String) -> AssemblyResult<()> {
		writeln!(output, "Memory Usage").unwrap();
		writeln!(output, "============").unwrap();

		// Calculate statistics from listing lines
		let mut code_bytes = 0;
		let mut lowest_addr = u16::MAX;
		let mut highest_addr = 0u16;

		for line in &self.lines {
			if let Some(addr) = line.address {
				code_bytes += line.bytes.len();
				lowest_addr = lowest_addr.min(addr);
				highest_addr = highest_addr.max(addr + line.bytes.len() as u16 - 1);
			}
		}

		if code_bytes > 0 {
			writeln!(output, "Code bytes generated: {}", code_bytes).unwrap();
			writeln!(output, "Address range: ${:04X} - ${:04X}", lowest_addr, highest_addr)
				.unwrap();
			writeln!(output, "Total range: {} bytes", (highest_addr - lowest_addr + 1)).unwrap();
		} else {
			writeln!(output, "No code generated").unwrap();
		}

		Ok(())
	}

	/// Write cross-references
	fn write_cross_references(&self, output: &mut String) -> AssemblyResult<()> {
		writeln!(output, "Cross References").unwrap();
		writeln!(output, "================").unwrap();
		writeln!(output, "(Cross-reference table would be populated with symbol usage)").unwrap();

		Ok(())
	}

	/// Get source line from file
	fn get_source_line(&self, file_path: &Path, line_number: usize) -> String {
		if let Some(lines) = self.source_files.get(file_path) {
			if line_number > 0 && line_number <= lines.len() {
				lines[line_number - 1].clone()
			} else {
				"(invalid line number)".to_string()
			}
		} else {
			"(source not available)".to_string()
		}
	}

	/// Clear all lines (for reuse)
	pub fn clear(&mut self) {
		self.lines.clear();
		self.current_line = 0;
		self.current_address = None;
		self.macro_level = 0;
	}

	/// Get number of lines
	pub fn line_count(&self) -> usize {
		self.lines.len()
	}

	/// Check if empty
	pub fn is_empty(&self) -> bool {
		self.lines.is_empty()
	}
}

impl Default for ListingGenerator {
	fn default() -> Self {
		Self::new()
	}
}

/// Utility functions for listing generation
pub mod utils {
	use super::*;

	/// Format a byte sequence as hex string
	pub fn format_bytes(bytes: &[u8], max_bytes: usize) -> String {
		let mut output = String::new();
		for (i, byte) in bytes.iter().take(max_bytes).enumerate() {
			if i > 0 {
				output.push(' ');
			}
			write!(output, "{:02X}", byte).unwrap();
		}
		output
	}

	/// Format an address as hex string
	pub fn format_address(addr: u16) -> String {
		format!("{:04X}", addr)
	}

	/// Truncate source line to maximum width
	pub fn truncate_source(source: &str, max_width: usize) -> String {
		if source.len() <= max_width {
			source.to_string()
		} else {
			let mut truncated = source[..max_width.saturating_sub(3)].to_string();
			truncated.push_str("...");
			truncated
		}
	}

	/// Escape special characters in source line
	pub fn escape_source(source: &str) -> String {
		source
			.chars()
			.map(|c| match c {
				'\t' => ' ',                // Convert tabs to spaces
				c if c.is_control() => '?', // Replace control chars
				c => c,
			})
			.collect()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::parsing::Statement;
	use std::path::PathBuf;

	#[test]
	fn test_listing_level_conversion() {
		assert_eq!(ListingLevel::from(0), ListingLevel::None);
		assert_eq!(ListingLevel::from(1), ListingLevel::Basic);
		assert_eq!(ListingLevel::from(2), ListingLevel::Standard);
		assert_eq!(ListingLevel::from(3), ListingLevel::Detailed);
		assert_eq!(ListingLevel::from(99), ListingLevel::Standard);
	}

	#[test]
	fn test_listing_line_creation() {
		let line = ListingLine::new(
			42,
			Some(0x8000),
			vec![0xA9, 0x00],
			"LDA #$00".to_string(),
			PathBuf::from("test.asm"),
		);

		assert_eq!(line.line_number, 42);
		assert_eq!(line.address, Some(0x8000));
		assert_eq!(line.bytes, vec![0xA9, 0x00]);
		assert_eq!(line.source, "LDA #$00");
		assert!(!line.has_error);
		assert!(!line.is_macro_expansion);
	}

	#[test]
	fn test_listing_line_with_error() {
		let line =
			ListingLine::new(1, None, Vec::new(), "INVALID".to_string(), PathBuf::from("test.asm"))
				.with_error("Unknown instruction".to_string());

		assert!(line.has_error);
		assert_eq!(line.error_message, Some("Unknown instruction".to_string()));
	}

	#[test]
	fn test_listing_line_format_basic() {
		let line = ListingLine::new(
			1,
			Some(0x8000),
			vec![0xA9, 0x00],
			"LDA #$00".to_string(),
			PathBuf::from("test.asm"),
		);

		let formatted = line.format(ListingLevel::Basic, false);
		assert!(formatted.contains("8000"));
		assert!(formatted.contains("A9 00"));
	}

	#[test]
	fn test_listing_line_format_standard() {
		let line = ListingLine::new(
			42,
			Some(0x8000),
			vec![0xA9, 0x00],
			"LDA #$00".to_string(),
			PathBuf::from("test.asm"),
		);

		let formatted = line.format(ListingLevel::Standard, false);
		assert!(formatted.contains("42"));
		assert!(formatted.contains("8000"));
		assert!(formatted.contains("A9 00"));
		assert!(formatted.contains("LDA #$00"));
	}

	#[test]
	fn test_listing_line_continuation() {
		let line = ListingLine::new(
			1,
			Some(0x8000),
			vec![0x01, 0x02, 0x03, 0x04, 0x05], // 5 bytes > 3
			"DCB $01,$02,$03,$04,$05".to_string(),
			PathBuf::from("test.asm"),
		);

		assert!(line.needs_continuation());
		let continuations = line.continuation_lines();
		assert_eq!(continuations.len(), 1);
		assert!(continuations[0].contains("04 05"));
	}

	#[test]
	fn test_listing_generator_creation() {
		let generator = ListingGenerator::new();
		assert_eq!(generator.config.level, ListingLevel::Standard);
		assert!(generator.lines.is_empty());
		assert!(generator.source_files.is_empty());
	}

	#[test]
	fn test_listing_generator_add_source() {
		let mut generator = ListingGenerator::new();
		let content = "LDA #$00\nSTA $2000\n";
		generator.add_source_file(PathBuf::from("test.asm"), content);

		assert!(generator.source_files.contains_key(&PathBuf::from("test.asm")));
		let lines = &generator.source_files[&PathBuf::from("test.asm")];
		assert_eq!(lines.len(), 2);
		assert_eq!(lines[0], "LDA #$00");
		assert_eq!(lines[1], "STA $2000");
	}

	#[test]
	fn test_listing_config_default() {
		let config = ListingConfig::default();
		assert_eq!(config.level, ListingLevel::Standard);
		assert!(!config.show_macro_expansions);
		assert!(!config.show_file_names);
		assert_eq!(config.max_source_width, 80);
		assert!(config.include_memory_stats);
		assert!(config.include_symbol_table);
	}

	#[test]
	fn test_utils_format_bytes() {
		assert_eq!(utils::format_bytes(&[0xA9, 0x00], 3), "A9 00");
		assert_eq!(utils::format_bytes(&[0x01, 0x02, 0x03, 0x04], 2), "01 02");
		assert_eq!(utils::format_bytes(&[], 3), "");
	}

	#[test]
	fn test_utils_format_address() {
		assert_eq!(utils::format_address(0x8000), "8000");
		assert_eq!(utils::format_address(0x0042), "0042");
	}

	#[test]
	fn test_utils_truncate_source() {
		assert_eq!(utils::truncate_source("short", 10), "short");
		assert_eq!(utils::truncate_source("this is a very long line", 10), "this i...");
	}

	#[test]
	fn test_utils_escape_source() {
		assert_eq!(utils::escape_source("normal text"), "normal text");
		assert_eq!(utils::escape_source("text\twith\ttabs"), "text with tabs");
		assert_eq!(utils::escape_source("text\nwith\ncontrol"), "text?with?control");
	}
}
