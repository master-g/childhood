//! Symbol table core functionality for the NES assembler.
//!
//! This module provides the core symbol table implementation with efficient
//! symbol storage, lookup, and management capabilities.

use crate::error::{AssemblyError, AssemblyResult, SourcePos};
use crate::symbols::{SymbolAttributes, SymbolStatus, SymbolVisibility};
use std::collections::HashMap;
use std::fmt;

/// Symbol value types supported by the assembler
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolValue {
	/// Numeric value (constants, expressions)
	Number(i32),
	/// Memory address
	Address(u16),
	/// String value (for string constants)
	String(String),
	/// Undefined value (forward reference)
	Undefined,
	/// Expression to be evaluated
	Expression(String),
}

impl SymbolValue {
	/// Convert to numeric value if possible
	pub fn as_number(&self) -> Option<i32> {
		match self {
			Self::Number(val) => Some(*val),
			Self::Address(addr) => Some(*addr as i32),
			_ => None,
		}
	}

	/// Convert to address if possible
	pub fn as_address(&self) -> Option<u16> {
		match self {
			Self::Address(addr) => Some(*addr),
			Self::Number(val) if *val >= 0 && *val <= 0xFFFF => Some(*val as u16),
			_ => None,
		}
	}

	/// Convert to string if possible
	pub fn as_string(&self) -> Option<&str> {
		match self {
			Self::String(s) => Some(s),
			_ => None,
		}
	}

	/// Check if value is defined
	pub fn is_defined(&self) -> bool {
		!matches!(self, Self::Undefined | Self::Expression(_))
	}

	/// Check if value is numeric
	pub fn is_numeric(&self) -> bool {
		matches!(self, Self::Number(_) | Self::Address(_))
	}

	/// Get the size in bytes this value would occupy
	pub fn size_hint(&self) -> usize {
		match self {
			Self::Number(val) => {
				if *val >= -128 && *val <= 255 {
					1
				} else {
					2
				}
			}
			Self::Address(_) => 2,
			Self::String(s) => s.len(),
			Self::Undefined | Self::Expression(_) => 0,
		}
	}
}

impl fmt::Display for SymbolValue {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Number(val) => write!(f, "{}", val),
			Self::Address(addr) => write!(f, "${:04X}", addr),
			Self::String(s) => write!(f, "\"{}\"", s),
			Self::Undefined => write!(f, "<undefined>"),
			Self::Expression(expr) => write!(f, "({})", expr),
		}
	}
}

/// Symbol type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolType {
	/// Label (address marker)
	Label,
	/// Variable (mutable value)
	Variable,
	/// Constant (immutable value)
	Constant,
	/// Macro definition
	Macro,
	/// Function/subroutine
	Function,
	/// Data definition
	Data,
	/// Section/segment marker
	Section,
	/// Equate (alias for another symbol)
	Equate,
}

impl SymbolType {
	/// Check if this symbol type represents an address
	pub fn is_address_type(&self) -> bool {
		matches!(self, Self::Label | Self::Function | Self::Data | Self::Section)
	}

	/// Check if this symbol type can be modified
	pub fn is_mutable(&self) -> bool {
		matches!(self, Self::Variable)
	}

	/// Check if this symbol type is executable
	pub fn is_executable(&self) -> bool {
		matches!(self, Self::Function | Self::Macro)
	}

	/// Get the default visibility for this symbol type
	pub fn default_visibility(&self) -> SymbolVisibility {
		match self {
			Self::Label | Self::Function => SymbolVisibility::Global,
			Self::Variable | Self::Constant => SymbolVisibility::Local,
			Self::Macro => SymbolVisibility::Local,
			Self::Data | Self::Section => SymbolVisibility::Global,
			Self::Equate => SymbolVisibility::Local,
		}
	}
}

impl fmt::Display for SymbolType {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let name = match self {
			Self::Label => "label",
			Self::Variable => "variable",
			Self::Constant => "constant",
			Self::Macro => "macro",
			Self::Function => "function",
			Self::Data => "data",
			Self::Section => "section",
			Self::Equate => "equate",
		};
		write!(f, "{}", name)
	}
}

/// Complete symbol information
#[derive(Debug, Clone)]
pub struct SymbolInfo {
	/// Symbol type
	symbol_type: SymbolType,
	/// Symbol value
	value: SymbolValue,
	/// Source position where defined
	pos: SourcePos,
	/// Scope ID where symbol is defined
	scope_id: usize,
	/// Symbol visibility
	visibility: SymbolVisibility,
	/// Symbol status
	status: SymbolStatus,
	/// Symbol attributes
	attributes: SymbolAttributes,
}

impl SymbolInfo {
	/// Create a new symbol info
	pub fn new(
		symbol_type: SymbolType,
		value: SymbolValue,
		pos: SourcePos,
		scope_id: usize,
	) -> Self {
		let visibility = symbol_type.default_visibility();
		let status = if value.is_defined() {
			SymbolStatus::Defined
		} else {
			SymbolStatus::Undefined
		};

		Self {
			symbol_type,
			value,
			pos,
			scope_id,
			visibility,
			status,
			attributes: SymbolAttributes::new(),
		}
	}

	/// Create a label symbol
	pub fn label(name: &str, address: u16, pos: SourcePos, scope_id: usize) -> Self {
		Self::new(SymbolType::Label, SymbolValue::Address(address), pos, scope_id)
	}

	/// Create a constant symbol
	pub fn constant(value: i32, pos: SourcePos, scope_id: usize) -> Self {
		let mut symbol = Self::new(SymbolType::Constant, SymbolValue::Number(value), pos, scope_id);
		symbol.attributes.is_constant = true;
		symbol
	}

	/// Create a variable symbol
	pub fn variable(value: i32, pos: SourcePos, scope_id: usize) -> Self {
		Self::new(SymbolType::Variable, SymbolValue::Number(value), pos, scope_id)
	}

	/// Create an undefined symbol (forward reference)
	pub fn undefined(symbol_type: SymbolType, pos: SourcePos, scope_id: usize) -> Self {
		Self::new(symbol_type, SymbolValue::Undefined, pos, scope_id)
	}

	// Getters
	pub fn symbol_type(&self) -> SymbolType {
		self.symbol_type
	}

	pub fn value(&self) -> &SymbolValue {
		&self.value
	}

	pub fn pos(&self) -> &SourcePos {
		&self.pos
	}

	pub fn scope_id(&self) -> usize {
		self.scope_id
	}

	pub fn visibility(&self) -> SymbolVisibility {
		self.visibility
	}

	pub fn status(&self) -> SymbolStatus {
		self.status
	}

	pub fn attributes(&self) -> &SymbolAttributes {
		&self.attributes
	}

	pub fn attributes_mut(&mut self) -> &mut SymbolAttributes {
		&mut self.attributes
	}

	// Setters
	pub fn set_value(&mut self, value: SymbolValue) {
		let is_defined = value.is_defined();
		self.value = value;
		self.status = if is_defined {
			SymbolStatus::Defined
		} else {
			SymbolStatus::Undefined
		};
	}

	pub fn set_visibility(&mut self, visibility: SymbolVisibility) {
		self.visibility = visibility;
	}

	pub fn set_status(&mut self, status: SymbolStatus) {
		self.status = status;
	}

	// State checks
	pub fn is_defined(&self) -> bool {
		self.status == SymbolStatus::Defined
	}

	pub fn is_constant(&self) -> bool {
		self.attributes.is_constant
	}

	pub fn is_referenced(&self) -> bool {
		self.attributes.is_referenced
	}

	pub fn can_redefine(&self) -> bool {
		self.attributes.is_redefinable
	}

	pub fn is_address(&self) -> bool {
		matches!(self.value, SymbolValue::Address(_)) || self.symbol_type.is_address_type()
	}

	pub fn is_numeric(&self) -> bool {
		self.value.is_numeric()
	}

	/// Mark symbol as referenced
	pub fn mark_referenced(&mut self) {
		self.attributes.mark_referenced();
	}

	/// Validate symbol definition
	pub fn validate(&self) -> AssemblyResult<()> {
		// Check if constant symbol has a defined value
		if self.is_constant() && !self.value.is_defined() {
			return Err(AssemblyError::symbol(
				self.pos.clone(),
				"Constant symbol must have a defined value".to_string(),
			));
		}

		// Check if address symbols have valid addresses
		if self.symbol_type.is_address_type() {
			if let Some(addr) = self.value.as_address() {
				// Validate address range (could be platform-specific)
				if addr > 0xFFFF {
					return Err(AssemblyError::invalid_address(
						self.pos.clone(),
						addr,
						"Address out of range".to_string(),
					));
				}
			}
		}

		Ok(())
	}
}

/// Symbol table for storing and managing symbols
#[derive(Debug, Clone)]
pub struct SymbolTable {
	/// Symbol storage
	symbols: HashMap<String, SymbolInfo>,
	/// Case sensitivity setting
	case_sensitive: bool,
}

impl SymbolTable {
	/// Create a new symbol table
	pub fn new() -> Self {
		Self {
			symbols: HashMap::new(),
			case_sensitive: true,
		}
	}

	/// Create a new case-insensitive symbol table
	pub fn new_case_insensitive() -> Self {
		Self {
			symbols: HashMap::new(),
			case_sensitive: false,
		}
	}

	/// Normalize symbol name according to case sensitivity
	fn normalize_name(&self, name: &str) -> String {
		if self.case_sensitive {
			name.to_string()
		} else {
			name.to_uppercase()
		}
	}

	/// Insert a symbol into the table
	pub fn insert(&mut self, name: String, symbol: SymbolInfo) -> AssemblyResult<()> {
		let normalized_name = self.normalize_name(&name);

		// Validate symbol before insertion
		symbol.validate()?;

		// Check for redefinition
		if let Some(existing) = self.symbols.get(&normalized_name) {
			if !existing.can_redefine() {
				return Err(AssemblyError::DuplicateSymbol {
					pos: symbol.pos.clone(),
					symbol: name,
					previous_pos: existing.pos.clone(),
				});
			}
		}

		self.symbols.insert(normalized_name, symbol);
		Ok(())
	}

	/// Get a symbol from the table
	pub fn get(&self, name: &str) -> Option<&SymbolInfo> {
		let normalized_name = self.normalize_name(name);
		self.symbols.get(&normalized_name)
	}

	/// Get a mutable symbol from the table
	pub fn get_mut(&mut self, name: &str) -> Option<&mut SymbolInfo> {
		let normalized_name = self.normalize_name(name);
		self.symbols.get_mut(&normalized_name)
	}

	/// Check if a symbol exists
	pub fn contains(&self, name: &str) -> bool {
		let normalized_name = self.normalize_name(name);
		self.symbols.contains_key(&normalized_name)
	}

	/// Remove a symbol from the table
	pub fn remove(&mut self, name: &str) -> Option<SymbolInfo> {
		let normalized_name = self.normalize_name(name);
		self.symbols.remove(&normalized_name)
	}

	/// Get all symbols
	pub fn symbols(&self) -> &HashMap<String, SymbolInfo> {
		&self.symbols
	}

	/// Get number of symbols
	pub fn len(&self) -> usize {
		self.symbols.len()
	}

	/// Check if table is empty
	pub fn is_empty(&self) -> bool {
		self.symbols.is_empty()
	}

	/// Clear all symbols
	pub fn clear(&mut self) {
		self.symbols.clear();
	}

	/// Get symbols by type
	pub fn symbols_by_type(&self, symbol_type: SymbolType) -> Vec<(&String, &SymbolInfo)> {
		self.symbols.iter().filter(|(_, symbol)| symbol.symbol_type() == symbol_type).collect()
	}

	/// Get symbols by scope
	pub fn symbols_by_scope(&self, scope_id: usize) -> Vec<(&String, &SymbolInfo)> {
		self.symbols.iter().filter(|(_, symbol)| symbol.scope_id() == scope_id).collect()
	}

	/// Get symbols by visibility
	pub fn symbols_by_visibility(
		&self,
		visibility: SymbolVisibility,
	) -> Vec<(&String, &SymbolInfo)> {
		self.symbols.iter().filter(|(_, symbol)| symbol.visibility() == visibility).collect()
	}

	/// Get undefined symbols
	pub fn undefined_symbols(&self) -> Vec<(&String, &SymbolInfo)> {
		self.symbols.iter().filter(|(_, symbol)| !symbol.is_defined()).collect()
	}

	/// Get unreferenced symbols
	pub fn unreferenced_symbols(&self) -> Vec<(&String, &SymbolInfo)> {
		self.symbols.iter().filter(|(_, symbol)| !symbol.is_referenced()).collect()
	}

	/// Lookup symbol value
	pub fn lookup_value(&self, name: &str) -> Option<&SymbolValue> {
		self.get(name).map(|symbol| symbol.value())
	}

	/// Lookup symbol as address
	pub fn lookup_address(&self, name: &str) -> Option<u16> {
		self.get(name).and_then(|symbol| symbol.value().as_address())
	}

	/// Lookup symbol as number
	pub fn lookup_number(&self, name: &str) -> Option<i32> {
		self.get(name).and_then(|symbol| symbol.value().as_number())
	}

	/// Mark symbol as referenced
	pub fn mark_referenced(&mut self, name: &str) -> bool {
		if let Some(symbol) = self.get_mut(name) {
			symbol.mark_referenced();
			true
		} else {
			false
		}
	}

	/// Define or update a symbol
	pub fn define_symbol(
		&mut self,
		name: String,
		symbol_type: SymbolType,
		value: SymbolValue,
		pos: SourcePos,
		scope_id: usize,
	) -> AssemblyResult<()> {
		let symbol = SymbolInfo::new(symbol_type, value, pos, scope_id);
		self.insert(name, symbol)
	}

	/// Import symbols from another table
	pub fn import_from(&mut self, other: &SymbolTable) -> AssemblyResult<usize> {
		let mut imported_count = 0;

		for (name, symbol) in &other.symbols {
			if symbol.visibility() == SymbolVisibility::Export {
				let mut imported_symbol = symbol.clone();
				imported_symbol.set_visibility(SymbolVisibility::Import);

				// Try to insert, skip if already exists
				if !self.contains(name) {
					self.insert(name.clone(), imported_symbol)?;
					imported_count += 1;
				}
			}
		}

		Ok(imported_count)
	}

	/// Export symbols to a new table
	pub fn export_symbols(&self, visibility: SymbolVisibility) -> SymbolTable {
		let mut exported = SymbolTable::new();
		exported.case_sensitive = self.case_sensitive;

		for (name, symbol) in &self.symbols {
			if symbol.visibility() == visibility {
				exported.symbols.insert(name.clone(), symbol.clone());
			}
		}

		exported
	}

	/// Generate symbol listing
	pub fn generate_listing(&self) -> String {
		let mut listing = String::new();
		listing.push_str("Symbol Table Listing:\n");
		listing.push_str("=====================\n\n");

		let mut sorted_symbols: Vec<_> = self.symbols.iter().collect();
		sorted_symbols.sort_by_key(|(name, _)| name.as_str());

		for (name, symbol) in sorted_symbols {
			listing.push_str(&format!(
				"{:<20} {:8} {:15} {:8} {}\n",
				name,
				symbol.symbol_type(),
				symbol.value(),
				symbol.visibility(),
				if symbol.is_referenced() {
					"REF"
				} else {
					""
				}
			));
		}

		listing
	}

	/// Validate all symbols in the table
	pub fn validate_all(&self) -> AssemblyResult<()> {
		for (name, symbol) in &self.symbols {
			symbol.validate().map_err(|mut err| {
				// Add symbol name to error context
				if let AssemblyError::Symbol {
					message,
					..
				} = &mut err
				{
					*message = format!("{}: {}", name, message);
				}
				err
			})?;
		}
		Ok(())
	}
}

impl Default for SymbolTable {
	fn default() -> Self {
		Self::new()
	}
}

impl SymbolTable {
	/// Define a constant symbol with a numeric value
	pub fn define_constant(&mut self, name: String, value: i32) -> AssemblyResult<()> {
		let symbol_info = SymbolInfo::new(
			SymbolType::Constant,
			SymbolValue::Number(value),
			SourcePos::file_only(std::path::PathBuf::from("<constants>")),
			0, // scope_id for global constants
		);
		self.insert(name, symbol_info)
	}
}

/// Symbol table statistics
#[derive(Debug, Clone)]
pub struct SymbolTableStats {
	pub total_symbols: usize,
	pub defined_symbols: usize,
	pub undefined_symbols: usize,
	pub referenced_symbols: usize,
	pub unreferenced_symbols: usize,
	pub symbols_by_type: HashMap<SymbolType, usize>,
	pub symbols_by_visibility: HashMap<SymbolVisibility, usize>,
}

impl SymbolTable {
	/// Generate statistics about the symbol table
	pub fn statistics(&self) -> SymbolTableStats {
		let mut stats = SymbolTableStats {
			total_symbols: self.symbols.len(),
			defined_symbols: 0,
			undefined_symbols: 0,
			referenced_symbols: 0,
			unreferenced_symbols: 0,
			symbols_by_type: HashMap::new(),
			symbols_by_visibility: HashMap::new(),
		};

		for symbol in self.symbols.values() {
			if symbol.is_defined() {
				stats.defined_symbols += 1;
			} else {
				stats.undefined_symbols += 1;
			}

			if symbol.is_referenced() {
				stats.referenced_symbols += 1;
			} else {
				stats.unreferenced_symbols += 1;
			}

			*stats.symbols_by_type.entry(symbol.symbol_type()).or_insert(0) += 1;
			*stats.symbols_by_visibility.entry(symbol.visibility()).or_insert(0) += 1;
		}

		stats
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::path::PathBuf;

	fn test_pos() -> SourcePos {
		SourcePos::new(PathBuf::from("test.asm"), 1, 1)
	}

	#[test]
	fn test_symbol_value() {
		let num = SymbolValue::Number(42);
		assert_eq!(num.as_number(), Some(42));
		assert!(num.is_numeric());
		assert!(num.is_defined());

		let addr = SymbolValue::Address(0x1000);
		assert_eq!(addr.as_address(), Some(0x1000));
		assert_eq!(addr.as_number(), Some(0x1000));

		let undef = SymbolValue::Undefined;
		assert!(!undef.is_defined());
	}

	#[test]
	fn test_symbol_type() {
		assert!(SymbolType::Label.is_address_type());
		assert!(!SymbolType::Variable.is_address_type());
		assert!(SymbolType::Variable.is_mutable());
		assert!(!SymbolType::Constant.is_mutable());
	}

	#[test]
	fn test_symbol_info() {
		let pos = test_pos();
		let symbol = SymbolInfo::label("test", 0x1000, pos, 0);

		assert_eq!(symbol.symbol_type(), SymbolType::Label);
		assert_eq!(symbol.value().as_address(), Some(0x1000));
		assert!(symbol.is_defined());
		assert!(symbol.is_address());
		assert!(!symbol.is_referenced());
	}

	#[test]
	fn test_symbol_table_basic() {
		let mut table = SymbolTable::new();
		let pos = test_pos();

		let symbol = SymbolInfo::label("test", 0x1000, pos, 0);
		assert!(table.insert("test".to_string(), symbol).is_ok());

		assert!(table.contains("test"));
		assert_eq!(table.len(), 1);

		let retrieved = table.get("test").unwrap();
		assert_eq!(retrieved.value().as_address(), Some(0x1000));
	}

	#[test]
	fn test_case_sensitivity() {
		let mut table = SymbolTable::new_case_insensitive();
		let pos = test_pos();

		let symbol = SymbolInfo::label("Test", 0x1000, pos, 0);
		table.insert("Test".to_string(), symbol).unwrap();

		// Should find with different case
		assert!(table.contains("test"));
		assert!(table.contains("TEST"));
		assert!(table.contains("Test"));
	}

	#[test]
	fn test_duplicate_symbol() {
		let mut table = SymbolTable::new();
		let pos = test_pos();

		let symbol1 = SymbolInfo::label("test", 0x1000, pos.clone(), 0);
		table.insert("test".to_string(), symbol1).unwrap();

		let symbol2 = SymbolInfo::label("test", 0x2000, pos, 0);
		assert!(table.insert("test".to_string(), symbol2).is_err());
	}

	#[test]
	fn test_symbol_lookup() {
		let mut table = SymbolTable::new();
		let pos = test_pos();

		let label = SymbolInfo::label("label", 0x1000, pos.clone(), 0);
		let constant = SymbolInfo::constant(42, pos, 0);

		table.insert("label".to_string(), label).unwrap();
		table.insert("constant".to_string(), constant).unwrap();

		assert_eq!(table.lookup_address("label"), Some(0x1000));
		assert_eq!(table.lookup_number("constant"), Some(42));
		assert_eq!(table.lookup_address("nonexistent"), None);
	}

	#[test]
	fn test_symbol_filtering() {
		let mut table = SymbolTable::new();
		let pos = test_pos();

		let label = SymbolInfo::label("label", 0x1000, pos.clone(), 0);
		let constant = SymbolInfo::constant(42, pos.clone(), 0);
		let mut variable = SymbolInfo::variable(100, pos, 0);
		variable.set_visibility(SymbolVisibility::Global);

		table.insert("label".to_string(), label).unwrap();
		table.insert("constant".to_string(), constant).unwrap();
		table.insert("variable".to_string(), variable).unwrap();

		let labels = table.symbols_by_type(SymbolType::Label);
		assert_eq!(labels.len(), 1);

		let globals = table.symbols_by_visibility(SymbolVisibility::Global);
		assert_eq!(globals.len(), 2); // label and variable
	}

	#[test]
	fn test_symbol_reference_tracking() {
		let mut table = SymbolTable::new();
		let pos = test_pos();

		let symbol = SymbolInfo::label("test", 0x1000, pos, 0);
		table.insert("test".to_string(), symbol).unwrap();

		assert!(!table.get("test").unwrap().is_referenced());

		table.mark_referenced("test");
		assert!(table.get("test").unwrap().is_referenced());
	}

	#[test]
	fn test_symbol_statistics() {
		let mut table = SymbolTable::new();
		let pos = test_pos();

		let label = SymbolInfo::label("label", 0x1000, pos.clone(), 0);
		let constant = SymbolInfo::constant(42, pos.clone(), 0);
		let undefined = SymbolInfo::undefined(SymbolType::Label, pos, 0);

		table.insert("label".to_string(), label).unwrap();
		table.insert("constant".to_string(), constant).unwrap();
		table.insert("undefined".to_string(), undefined).unwrap();

		table.mark_referenced("label");

		let stats = table.statistics();
		assert_eq!(stats.total_symbols, 3);
		assert_eq!(stats.defined_symbols, 2);
		assert_eq!(stats.undefined_symbols, 1);
		assert_eq!(stats.referenced_symbols, 1);
		assert_eq!(stats.unreferenced_symbols, 2);
	}
}
