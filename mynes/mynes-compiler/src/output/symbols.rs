//! Symbol export functionality for the NES assembler.
//!
//! This module provides functionality to export symbols in various formats,
//! including FCEUX-compatible symbol files for debugging NES programs.

use crate::error::{AssemblyError, AssemblyResult, SourcePos};
use crate::symbols::{SymbolManager, SymbolType, SymbolValue, SymbolVisibility};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Supported symbol export formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolFormat {
	/// FCEUX debugger format (.nl)
	Fceux,
	/// Mesen debugger format (.mlb)
	Mesen,
	/// No$NES debugger format (.sym)
	NoNes,
	/// Generic text format
	Text,
}

impl Default for SymbolFormat {
	fn default() -> Self {
		Self::Fceux
	}
}

/// Symbol export configuration
#[derive(Debug, Clone)]
pub struct SymbolExportConfig {
	/// Export format
	pub format: SymbolFormat,
	/// Bank offset for FCEUX compatibility
	pub bank_offset: u8,
	/// Include local symbols
	pub include_local: bool,
	/// Include global symbols
	pub include_global: bool,
	/// Include constants
	pub include_constants: bool,
	/// Include addresses
	pub include_addresses: bool,
	/// Sort symbols by name
	pub sort_by_name: bool,
	/// Sort symbols by address
	pub sort_by_address: bool,
	/// Filter by address range
	pub address_filter: Option<(u16, u16)>,
	/// Prefix for symbol names
	pub name_prefix: Option<String>,
}

impl Default for SymbolExportConfig {
	fn default() -> Self {
		Self {
			format: SymbolFormat::default(),
			bank_offset: 0,
			include_local: true,
			include_global: true,
			include_constants: true,
			include_addresses: true,
			sort_by_name: false,
			sort_by_address: true,
			address_filter: None,
			name_prefix: None,
		}
	}
}

/// A symbol entry for export
#[derive(Debug, Clone)]
pub struct ExportSymbol {
	/// Symbol name
	pub name: String,
	/// Symbol value
	pub value: SymbolValue,
	/// Symbol type
	pub symbol_type: SymbolType,
	/// Symbol visibility
	pub visibility: SymbolVisibility,
	/// Address (for address-based symbols)
	pub address: Option<u16>,
	/// Bank number (for banked symbols)
	pub bank: Option<u8>,
	/// Comment or description
	pub comment: Option<String>,
}

impl ExportSymbol {
	/// Create a new export symbol
	pub fn new(
		name: String,
		value: SymbolValue,
		symbol_type: SymbolType,
		visibility: SymbolVisibility,
	) -> Self {
		let address = match value {
			SymbolValue::Address(addr) => Some(addr),
			_ => None,
		};

		Self {
			name,
			value,
			symbol_type,
			visibility,
			address,
			bank: None,
			comment: None,
		}
	}

	/// Set bank number
	pub fn with_bank(mut self, bank: u8) -> Self {
		self.bank = Some(bank);
		self
	}

	/// Set comment
	pub fn with_comment(mut self, comment: String) -> Self {
		self.comment = Some(comment);
		self
	}

	/// Get effective address with bank offset
	pub fn effective_address(&self, bank_offset: u8) -> Option<u16> {
		self.address.map(|addr| {
			if let Some(bank) = self.bank {
				// Apply bank offset for FCEUX compatibility
				let effective_bank = bank.wrapping_add(bank_offset);
				(effective_bank as u16) << 12 | (addr & 0x0FFF)
			} else {
				addr
			}
		})
	}

	/// Format symbol for specific export format
	pub fn format(&self, format: SymbolFormat, config: &SymbolExportConfig) -> String {
		match format {
			SymbolFormat::Fceux => self.format_fceux(config),
			SymbolFormat::Mesen => self.format_mesen(config),
			SymbolFormat::NoNes => self.format_nones(config),
			SymbolFormat::Text => self.format_text(config),
		}
	}

	/// Format for FCEUX debugger (.nl file)
	fn format_fceux(&self, config: &SymbolExportConfig) -> String {
		if let Some(addr) = self.effective_address(config.bank_offset) {
			let name = if let Some(prefix) = &config.name_prefix {
				format!("{}{}", prefix, self.name)
			} else {
				self.name.clone()
			};

			// FCEUX format: $ADDRESS#SYMBOL#
			format!("${:04X}#{name}#", addr)
		} else if let SymbolValue::Number(num) = self.value {
			// Constants as equates
			let name = if let Some(prefix) = &config.name_prefix {
				format!("{}{}", prefix, self.name)
			} else {
				self.name.clone()
			};
			format!("${:04X}#{name}#", num as u16)
		} else {
			String::new()
		}
	}

	/// Format for Mesen debugger (.mlb file)
	fn format_mesen(&self, config: &SymbolExportConfig) -> String {
		if let Some(addr) = self.effective_address(config.bank_offset) {
			let name = if let Some(prefix) = &config.name_prefix {
				format!("{}{}", prefix, self.name)
			} else {
				self.name.clone()
			};

			let symbol_type = match self.symbol_type {
				SymbolType::Label => "CODE",
				SymbolType::Variable => "DATA",
				SymbolType::Constant => "CONST",
				SymbolType::Function => "FUNC",
				SymbolType::Macro => "MACRO",
				SymbolType::Data => "DATA",
				SymbolType::Section => "SECT",
				SymbolType::Equate => "EQU",
			};

			// Mesen format: TYPE:ADDRESS:NAME
			format!("{}:{:04X}:{}", symbol_type, addr, name)
		} else {
			String::new()
		}
	}

	/// Format for No$NES debugger (.sym file)
	fn format_nones(&self, config: &SymbolExportConfig) -> String {
		if let Some(addr) = self.effective_address(config.bank_offset) {
			let name = if let Some(prefix) = &config.name_prefix {
				format!("{}{}", prefix, self.name)
			} else {
				self.name.clone()
			};

			// No$NES format: ADDRESS NAME
			format!("{:04X} {}", addr, name)
		} else {
			String::new()
		}
	}

	/// Format for generic text format
	fn format_text(&self, config: &SymbolExportConfig) -> String {
		let name = if let Some(prefix) = &config.name_prefix {
			format!("{}{}", prefix, self.name)
		} else {
			self.name.clone()
		};

		let value_str = match &self.value {
			SymbolValue::Address(addr) => {
				if let Some(effective_addr) = self.effective_address(config.bank_offset) {
					format!("${:04X}", effective_addr)
				} else {
					format!("${:04X}", addr)
				}
			}
			SymbolValue::Number(num) => format!("{}", num),
			SymbolValue::String(s) => format!("\"{}\"", s),
			SymbolValue::Undefined => "UNDEFINED".to_string(),
			SymbolValue::Expression(expr) => format!("EXPR({})", expr),
		};

		let type_str = match self.symbol_type {
			SymbolType::Label => "Label",
			SymbolType::Variable => "Variable",
			SymbolType::Constant => "Constant",
			SymbolType::Function => "Function",
			SymbolType::Macro => "Macro",
			SymbolType::Data => "Data",
			SymbolType::Section => "Section",
			SymbolType::Equate => "Equate",
		};

		let visibility_str = match self.visibility {
			SymbolVisibility::Local => "Local",
			SymbolVisibility::Global => "Global",
			SymbolVisibility::Export => "Export",
			SymbolVisibility::Import => "Import",
		};

		if let Some(comment) = &self.comment {
			format!(
				"{:<20} {:<8} {:<8} {:<8} ; {}",
				name, value_str, type_str, visibility_str, comment
			)
		} else {
			format!("{:<20} {:<8} {:<8} {:<8}", name, value_str, type_str, visibility_str)
		}
	}
}

/// Symbol exporter
#[derive(Debug)]
pub struct SymbolExporter {
	/// Export configuration
	config: SymbolExportConfig,
	/// Collected symbols for export
	symbols: Vec<ExportSymbol>,
}

impl SymbolExporter {
	/// Create a new symbol exporter
	pub fn new() -> Self {
		Self {
			config: SymbolExportConfig::default(),
			symbols: Vec::new(),
		}
	}

	/// Create with custom configuration
	pub fn with_config(config: SymbolExportConfig) -> Self {
		Self {
			config,
			symbols: Vec::new(),
		}
	}

	/// Add symbols from symbol manager
	pub fn add_symbols_from_manager(&mut self, symbol_manager: &SymbolManager) {
		let symbol_table = symbol_manager.symbol_table();

		for (name, symbol_info) in symbol_table.symbols() {
			// Apply filters
			if !self.should_include_symbol(&symbol_info.symbol_type(), &symbol_info.visibility()) {
				continue;
			}

			// Apply address filter
			if let Some((min_addr, max_addr)) = self.config.address_filter {
				if let SymbolValue::Address(addr) = symbol_info.value() {
					if *addr < min_addr || *addr > max_addr {
						continue;
					}
				}
			}

			let export_symbol = ExportSymbol::new(
				name.clone(),
				symbol_info.value().clone(),
				symbol_info.symbol_type(),
				symbol_info.visibility(),
			);

			self.symbols.push(export_symbol);
		}
	}

	/// Add a single symbol
	pub fn add_symbol(&mut self, symbol: ExportSymbol) {
		if self.should_include_symbol(&symbol.symbol_type, &symbol.visibility) {
			self.symbols.push(symbol);
		}
	}

	/// Generate symbol file
	pub fn generate(&self, output_path: &Path) -> AssemblyResult<()> {
		let content = self.format_symbols()?;

		let file = File::create(output_path).map_err(|e| AssemblyError::Io {
			pos: Some(SourcePos::file_only(output_path.to_path_buf())),
			source: e,
		})?;

		let mut writer = BufWriter::new(file);
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

	/// Generate multiple symbol files with different formats
	pub fn generate_multiple(
		&self,
		base_path: &Path,
		formats: &[SymbolFormat],
	) -> AssemblyResult<()> {
		for &format in formats {
			let mut config = self.config.clone();
			config.format = format;

			let exporter = SymbolExporter::with_config(config);
			let mut symbols = self.symbols.clone();
			exporter.sort_symbols(&mut symbols);

			let extension = match format {
				SymbolFormat::Fceux => "nl",
				SymbolFormat::Mesen => "mlb",
				SymbolFormat::NoNes => "sym",
				SymbolFormat::Text => "txt",
			};

			let output_path = base_path.with_extension(extension);
			let content = exporter.format_symbols_with_list(&symbols)?;

			let file = File::create(&output_path).map_err(|e| AssemblyError::Io {
				pos: Some(SourcePos::file_only(output_path.clone())),
				source: e,
			})?;

			let mut writer = BufWriter::new(file);
			writer.write_all(content.as_bytes()).map_err(|e| AssemblyError::Io {
				pos: Some(SourcePos::file_only(output_path.clone())),
				source: e,
			})?;

			writer.flush().map_err(|e| AssemblyError::Io {
				pos: Some(SourcePos::file_only(output_path)),
				source: e,
			})?;
		}

		Ok(())
	}

	/// Format symbols as string
	fn format_symbols(&self) -> AssemblyResult<String> {
		let mut symbols = self.symbols.clone();
		self.sort_symbols(&mut symbols);
		self.format_symbols_with_list(&symbols)
	}

	/// Format symbols with provided list
	fn format_symbols_with_list(&self, symbols: &[ExportSymbol]) -> AssemblyResult<String> {
		let mut output = String::new();

		// Add header for text format
		if self.config.format == SymbolFormat::Text {
			output.push_str("Symbol Table Export\n");
			output.push_str("===================\n");
			output.push_str("NAME                 VALUE    TYPE     SCOPE\n");
			output.push_str("-------------------- -------- -------- --------\n");
		}

		// Format each symbol
		for symbol in symbols {
			let formatted = symbol.format(self.config.format, &self.config);
			if !formatted.is_empty() {
				output.push_str(&formatted);
				output.push('\n');
			}
		}

		// Add footer statistics for text format
		if self.config.format == SymbolFormat::Text {
			output.push('\n');
			output.push_str(&format!("Total symbols exported: {}\n", symbols.len()));

			// Count by type
			let mut type_counts = HashMap::new();
			for symbol in symbols {
				*type_counts.entry(symbol.symbol_type).or_insert(0) += 1;
			}

			for (symbol_type, count) in type_counts {
				output.push_str(&format!("  {:?}: {}\n", symbol_type, count));
			}
		}

		Ok(output)
	}

	/// Sort symbols according to configuration
	fn sort_symbols(&self, symbols: &mut [ExportSymbol]) {
		if self.config.sort_by_name {
			symbols.sort_by(|a, b| a.name.cmp(&b.name));
		} else if self.config.sort_by_address {
			symbols.sort_by(|a, b| {
				match (
					a.effective_address(self.config.bank_offset),
					b.effective_address(self.config.bank_offset),
				) {
					(Some(addr_a), Some(addr_b)) => addr_a.cmp(&addr_b),
					(Some(_), None) => std::cmp::Ordering::Less,
					(None, Some(_)) => std::cmp::Ordering::Greater,
					(None, None) => a.name.cmp(&b.name),
				}
			});
		}
	}

	/// Check if symbol should be included based on filters
	fn should_include_symbol(
		&self,
		symbol_type: &SymbolType,
		visibility: &SymbolVisibility,
	) -> bool {
		// Check type filters
		match symbol_type {
			SymbolType::Constant => {
				if !self.config.include_constants {
					return false;
				}
			}
			SymbolType::Label | SymbolType::Function | SymbolType::Variable => {
				if !self.config.include_addresses {
					return false;
				}
			}
			SymbolType::Macro | SymbolType::Data | SymbolType::Section | SymbolType::Equate => {
				// Include these types by default
			}
		}

		// Check visibility filters
		match visibility {
			SymbolVisibility::Local => self.config.include_local,
			SymbolVisibility::Global | SymbolVisibility::Export | SymbolVisibility::Import => {
				self.config.include_global
			}
		}
	}

	/// Clear all symbols
	pub fn clear(&mut self) {
		self.symbols.clear();
	}

	/// Get symbol count
	pub fn symbol_count(&self) -> usize {
		self.symbols.len()
	}

	/// Get symbols by type
	pub fn symbols_by_type(&self, symbol_type: SymbolType) -> Vec<&ExportSymbol> {
		self.symbols.iter().filter(|s| s.symbol_type == symbol_type).collect()
	}

	/// Get symbols in address range
	pub fn symbols_in_range(&self, start: u16, end: u16) -> Vec<&ExportSymbol> {
		self.symbols
			.iter()
			.filter(|s| {
				if let Some(addr) = s.effective_address(self.config.bank_offset) {
					addr >= start && addr <= end
				} else {
					false
				}
			})
			.collect()
	}
}

impl Default for SymbolExporter {
	fn default() -> Self {
		Self::new()
	}
}

/// Utility functions for symbol export
pub mod utils {
	use super::*;

	/// Create FCEUX-compatible symbol file name
	pub fn fceux_filename(base_name: &str) -> String {
		format!("{}.nl", base_name)
	}

	/// Create Mesen-compatible symbol file name
	pub fn mesen_filename(base_name: &str) -> String {
		format!("{}.mlb", base_name)
	}

	/// Create No$NES-compatible symbol file name
	pub fn nones_filename(base_name: &str) -> String {
		format!("{}.sym", base_name)
	}

	/// Validate symbol name for export
	pub fn is_valid_symbol_name(name: &str) -> bool {
		if name.is_empty() {
			return false;
		}

		// First character must be letter or underscore
		let first_char = name.chars().next().unwrap();
		if !first_char.is_ascii_alphabetic() && first_char != '_' {
			return false;
		}

		// Remaining characters must be alphanumeric or underscore
		name.chars().skip(1).all(|c| c.is_ascii_alphanumeric() || c == '_')
	}

	/// Sanitize symbol name for export
	pub fn sanitize_symbol_name(name: &str) -> String {
		if name.is_empty() {
			return "_unnamed".to_string();
		}

		let mut result = String::new();

		// Handle first character
		let first_char = name.chars().next().unwrap();
		if first_char.is_ascii_alphabetic() || first_char == '_' {
			result.push(first_char);
		} else {
			result.push('_');
		}

		// Handle remaining characters
		for c in name.chars().skip(1) {
			if c.is_ascii_alphanumeric() || c == '_' {
				result.push(c);
			} else {
				result.push('_');
			}
		}

		result
	}

	/// Convert address to bank and offset
	pub fn address_to_bank_offset(address: u16) -> (u8, u16) {
		let bank = (address >> 12) as u8;
		let offset = address & 0x0FFF;
		(bank, offset)
	}

	/// Convert bank and offset to address
	pub fn bank_offset_to_address(bank: u8, offset: u16) -> u16 {
		((bank as u16) << 12) | (offset & 0x0FFF)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::symbols::{SymbolType, SymbolValue, SymbolVisibility};

	#[test]
	fn test_export_symbol_creation() {
		let symbol = ExportSymbol::new(
			"test_label".to_string(),
			SymbolValue::Address(0x8000),
			SymbolType::Label,
			SymbolVisibility::Global,
		);

		assert_eq!(symbol.name, "test_label");
		assert_eq!(symbol.address, Some(0x8000));
		assert_eq!(symbol.symbol_type, SymbolType::Label);
		assert_eq!(symbol.visibility, SymbolVisibility::Global);
	}

	#[test]
	fn test_export_symbol_with_bank() {
		let symbol = ExportSymbol::new(
			"banked_label".to_string(),
			SymbolValue::Address(0x8000),
			SymbolType::Label,
			SymbolVisibility::Global,
		)
		.with_bank(1);

		assert_eq!(symbol.bank, Some(1));
		assert_eq!(symbol.effective_address(0), Some(0x1000)); // Bank 1, offset 0x000
	}

	#[test]
	fn test_fceux_format() {
		let symbol = ExportSymbol::new(
			"start".to_string(),
			SymbolValue::Address(0x8000),
			SymbolType::Label,
			SymbolVisibility::Global,
		);

		let config = SymbolExportConfig::default();
		let formatted = symbol.format_fceux(&config);
		assert_eq!(formatted, "$8000#start#");
	}

	#[test]
	fn test_mesen_format() {
		let symbol = ExportSymbol::new(
			"main_loop".to_string(),
			SymbolValue::Address(0x8000),
			SymbolType::Label,
			SymbolVisibility::Global,
		);

		let config = SymbolExportConfig::default();
		let formatted = symbol.format_mesen(&config);
		assert_eq!(formatted, "CODE:8000:main_loop");
	}

	#[test]
	fn test_nones_format() {
		let symbol = ExportSymbol::new(
			"interrupt".to_string(),
			SymbolValue::Address(0xFFFA),
			SymbolType::Label,
			SymbolVisibility::Global,
		);

		let config = SymbolExportConfig::default();
		let formatted = symbol.format_nones(&config);
		assert_eq!(formatted, "FFFA interrupt");
	}

	#[test]
	fn test_symbol_exporter_creation() {
		let exporter = SymbolExporter::new();
		assert_eq!(exporter.config.format, SymbolFormat::Fceux);
		assert!(exporter.symbols.is_empty());
	}

	#[test]
	fn test_symbol_exporter_add_symbol() {
		let mut exporter = SymbolExporter::new();
		let symbol = ExportSymbol::new(
			"test".to_string(),
			SymbolValue::Address(0x8000),
			SymbolType::Label,
			SymbolVisibility::Global,
		);

		exporter.add_symbol(symbol);
		assert_eq!(exporter.symbol_count(), 1);
	}

	#[test]
	fn test_symbol_filtering() {
		let config = SymbolExportConfig {
			include_local: false,
			include_global: true,
			include_constants: false,
			include_addresses: true,
			..Default::default()
		};

		let exporter = SymbolExporter::with_config(config);

		// Should include global address symbol
		assert!(exporter.should_include_symbol(&SymbolType::Label, &SymbolVisibility::Global));

		// Should not include local symbol
		assert!(!exporter.should_include_symbol(&SymbolType::Label, &SymbolVisibility::Local));

		// Should not include constants
		assert!(!exporter.should_include_symbol(&SymbolType::Constant, &SymbolVisibility::Global));
	}

	#[test]
	fn test_utils_symbol_name_validation() {
		assert!(utils::is_valid_symbol_name("valid_name"));
		assert!(utils::is_valid_symbol_name("_underscore"));
		assert!(utils::is_valid_symbol_name("name123"));
		assert!(!utils::is_valid_symbol_name("123invalid"));
		assert!(!utils::is_valid_symbol_name(""));
		assert!(!utils::is_valid_symbol_name("invalid-name"));
	}

	#[test]
	fn test_utils_symbol_name_sanitization() {
		assert_eq!(utils::sanitize_symbol_name("valid_name"), "valid_name");
		assert_eq!(utils::sanitize_symbol_name("123invalid"), "_23invalid");
		assert_eq!(utils::sanitize_symbol_name("invalid-name"), "invalid_name");
		assert_eq!(utils::sanitize_symbol_name(""), "_unnamed");
	}

	#[test]
	fn test_utils_bank_conversion() {
		let (bank, offset) = utils::address_to_bank_offset(0x1234);
		assert_eq!(bank, 1);
		assert_eq!(offset, 0x234);

		let address = utils::bank_offset_to_address(bank, offset);
		assert_eq!(address, 0x1234);
	}

	#[test]
	fn test_utils_filename_generation() {
		assert_eq!(utils::fceux_filename("game"), "game.nl");
		assert_eq!(utils::mesen_filename("game"), "game.mlb");
		assert_eq!(utils::nones_filename("game"), "game.sym");
	}

	#[test]
	fn test_symbol_sorting() {
		let mut exporter = SymbolExporter::new();
		exporter.config.sort_by_address = true;

		let symbol1 = ExportSymbol::new(
			"second".to_string(),
			SymbolValue::Address(0x8010),
			SymbolType::Label,
			SymbolVisibility::Global,
		);

		let symbol2 = ExportSymbol::new(
			"first".to_string(),
			SymbolValue::Address(0x8000),
			SymbolType::Label,
			SymbolVisibility::Global,
		);

		exporter.symbols = vec![symbol1, symbol2];
		let mut symbols = exporter.symbols.clone();
		exporter.sort_symbols(&mut symbols);
		exporter.symbols = symbols;

		assert_eq!(exporter.symbols[0].name, "first");
		assert_eq!(exporter.symbols[1].name, "second");
	}

	#[test]
	fn test_symbols_in_range() {
		let mut exporter = SymbolExporter::new();

		let symbol1 = ExportSymbol::new(
			"in_range".to_string(),
			SymbolValue::Address(0x8010),
			SymbolType::Label,
			SymbolVisibility::Global,
		);

		let symbol2 = ExportSymbol::new(
			"out_of_range".to_string(),
			SymbolValue::Address(0x9000),
			SymbolType::Label,
			SymbolVisibility::Global,
		);

		exporter.symbols = vec![symbol1, symbol2];

		let in_range = exporter.symbols_in_range(0x8000, 0x8FFF);
		assert_eq!(in_range.len(), 1);
		assert_eq!(in_range[0].name, "in_range");
	}
}
