//! Symbol table management for the NES assembler.
//!
//! This module provides comprehensive symbol management including symbol definition,
//! resolution, scoping, and cross-reference tracking for the NES assembler.

pub mod resolver;
pub mod scope;
pub mod table;

// Re-exports for convenience
pub use resolver::{ForwardReference, ResolutionContext, SymbolResolver};
pub use scope::{ScopeManager, ScopeType, SymbolScope};
pub use table::{SymbolInfo, SymbolTable, SymbolType, SymbolValue};

// Type aliases for compatibility
pub type Symbol = SymbolInfo;
pub type SymbolLocation = crate::error::SourcePos;

use crate::error::{AssemblyError, AssemblyResult, SourcePos};
use std::collections::HashMap;
use std::fmt;

/// Symbol visibility levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolVisibility {
	/// Local symbol (visible only in current scope)
	Local,
	/// Global symbol (visible across all files)
	Global,
	/// Exported symbol (visible to other modules)
	Export,
	/// Imported symbol (from other modules)
	Import,
}

impl fmt::Display for SymbolVisibility {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Local => write!(f, "local"),
			Self::Global => write!(f, "global"),
			Self::Export => write!(f, "export"),
			Self::Import => write!(f, "import"),
		}
	}
}

/// Symbol status during assembly
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolStatus {
	/// Symbol is defined and has a value
	Defined,
	/// Symbol is declared but not yet defined
	Declared,
	/// Symbol is undefined (forward reference)
	Undefined,
	/// Symbol definition is pending (during evaluation)
	Pending,
}

impl fmt::Display for SymbolStatus {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Defined => write!(f, "defined"),
			Self::Declared => write!(f, "declared"),
			Self::Undefined => write!(f, "undefined"),
			Self::Pending => write!(f, "pending"),
		}
	}
}

/// Symbol attributes for additional metadata
#[derive(Debug, Clone, Default)]
pub struct SymbolAttributes {
	/// Whether the symbol is a constant
	pub is_constant: bool,
	/// Whether the symbol can be redefined
	pub is_redefinable: bool,
	/// Whether the symbol is automatically generated
	pub is_auto_generated: bool,
	/// Whether the symbol has been referenced
	pub is_referenced: bool,
	/// Symbol description/comment
	pub description: Option<String>,
	/// Custom attributes
	pub custom: HashMap<String, String>,
}

impl SymbolAttributes {
	/// Create new default attributes
	pub fn new() -> Self {
		Self::default()
	}

	/// Create attributes for a constant symbol
	pub fn constant() -> Self {
		Self {
			is_constant: true,
			..Default::default()
		}
	}

	/// Create attributes for a redefinable symbol
	pub fn redefinable() -> Self {
		Self {
			is_redefinable: true,
			..Default::default()
		}
	}

	/// Create attributes for an auto-generated symbol
	pub fn auto_generated() -> Self {
		Self {
			is_auto_generated: true,
			..Default::default()
		}
	}

	/// Mark symbol as referenced
	pub fn mark_referenced(&mut self) {
		self.is_referenced = true;
	}

	/// Add a custom attribute
	pub fn add_custom(&mut self, key: String, value: String) {
		self.custom.insert(key, value);
	}

	/// Get a custom attribute
	pub fn get_custom(&self, key: &str) -> Option<&String> {
		self.custom.get(key)
	}
}

/// Cross-reference information for symbols
#[derive(Debug, Clone)]
pub struct CrossReference {
	/// File where the reference occurs
	pub file: String,
	/// Line number of the reference
	pub line: usize,
	/// Column number of the reference
	pub column: usize,
	/// Type of reference (definition, usage, etc.)
	pub ref_type: ReferenceType,
	/// Context of the reference
	pub context: String,
}

impl CrossReference {
	/// Create a new cross-reference
	pub fn new(
		file: String,
		line: usize,
		column: usize,
		ref_type: ReferenceType,
		context: String,
	) -> Self {
		Self {
			file,
			line,
			column,
			ref_type,
			context,
		}
	}

	/// Create a cross-reference from a source position
	pub fn from_pos(pos: &SourcePos, ref_type: ReferenceType, context: String) -> Self {
		Self::new(pos.file.to_string_lossy().to_string(), pos.line, pos.column, ref_type, context)
	}
}

/// Types of symbol references
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceType {
	/// Symbol definition
	Definition,
	/// Symbol usage/reference
	Usage,
	/// Symbol assignment
	Assignment,
	/// Symbol declaration
	Declaration,
}

impl fmt::Display for ReferenceType {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Definition => write!(f, "definition"),
			Self::Usage => write!(f, "usage"),
			Self::Assignment => write!(f, "assignment"),
			Self::Declaration => write!(f, "declaration"),
		}
	}
}

/// Symbol definition context
#[derive(Debug, Clone)]
pub struct SymbolDefinition {
	/// Symbol name
	pub name: String,
	/// Symbol type
	pub symbol_type: SymbolType,
	/// Symbol value
	pub value: SymbolValue,
	/// Source position of definition
	pub pos: SourcePos,
	/// Symbol visibility
	pub visibility: SymbolVisibility,
	/// Symbol status
	pub status: SymbolStatus,
	/// Symbol attributes
	pub attributes: SymbolAttributes,
	/// Cross-references
	pub cross_refs: Vec<CrossReference>,
}

impl SymbolDefinition {
	/// Create a new symbol definition
	pub fn new(
		name: String,
		symbol_type: SymbolType,
		value: SymbolValue,
		pos: SourcePos,
		visibility: SymbolVisibility,
	) -> Self {
		Self {
			name,
			symbol_type,
			value,
			pos,
			visibility,
			status: SymbolStatus::Defined,
			attributes: SymbolAttributes::new(),
			cross_refs: Vec::new(),
		}
	}

	/// Add a cross-reference
	pub fn add_cross_ref(&mut self, cross_ref: CrossReference) {
		self.cross_refs.push(cross_ref);
	}

	/// Mark as referenced
	pub fn mark_referenced(&mut self) {
		self.attributes.mark_referenced();
	}

	/// Check if symbol is defined
	pub fn is_defined(&self) -> bool {
		self.status == SymbolStatus::Defined
	}

	/// Check if symbol is constant
	pub fn is_constant(&self) -> bool {
		self.attributes.is_constant
	}

	/// Check if symbol can be redefined
	pub fn can_redefine(&self) -> bool {
		self.attributes.is_redefinable
	}

	/// Get reference count
	pub fn reference_count(&self) -> usize {
		self.cross_refs.len()
	}
}

/// Symbol manager for high-level symbol operations
pub struct SymbolManager {
	/// Main symbol table
	table: SymbolTable,
	/// Scope manager
	scope_manager: ScopeManager,
	/// Symbol resolver
	resolver: SymbolResolver,
	/// Auto-generated symbol counter
	auto_symbol_counter: u32,
}

impl SymbolManager {
	/// Create a new symbol manager
	pub fn new() -> Self {
		Self {
			table: SymbolTable::new(),
			scope_manager: ScopeManager::new(),
			resolver: SymbolResolver::new(),
			auto_symbol_counter: 0,
		}
	}

	/// Define a new symbol
	pub fn define_symbol(
		&mut self,
		name: String,
		symbol_type: SymbolType,
		value: SymbolValue,
		pos: SourcePos,
		visibility: SymbolVisibility,
	) -> AssemblyResult<()> {
		// Check if symbol already exists
		if let Some(existing) = self.table.get(&name) {
			if !existing.can_redefine() {
				return Err(AssemblyError::DuplicateSymbol {
					pos,
					symbol: name,
					previous_pos: existing.pos().clone(),
				});
			}
		}

		// Create symbol definition
		let mut definition =
			SymbolDefinition::new(name.clone(), symbol_type, value, pos.clone(), visibility);

		// Add definition cross-reference
		definition.add_cross_ref(CrossReference::from_pos(
			&pos,
			ReferenceType::Definition,
			format!("Definition of {}", name),
		));

		// Create symbol info
		let symbol_info = SymbolInfo::new(
			symbol_type,
			definition.value.clone(),
			pos,
			self.scope_manager.current_scope_id(),
		);

		// Add to table
		self.table.insert(name, symbol_info)?;

		Ok(())
	}

	/// Resolve a symbol reference
	pub fn resolve_symbol(&mut self, name: &str, pos: &SourcePos) -> AssemblyResult<SymbolValue> {
		// Mark as referenced
		if let Some(symbol) = self.table.get_mut(name) {
			symbol.mark_referenced();
		}

		// Resolve the symbol
		match self.resolver.resolve(name, &self.table, &self.scope_manager) {
			Ok(Some(value)) => Ok(value),
			Ok(None) => {
				// Add forward reference
				self.resolver.add_forward_reference(name.to_string(), pos.clone());
				Err(AssemblyError::undefined_symbol(pos.clone(), name))
			}
			Err(_) => {
				// Add forward reference
				self.resolver.add_forward_reference(name.to_string(), pos.clone());
				Err(AssemblyError::undefined_symbol(pos.clone(), name))
			}
		}
	}

	/// Generate an auto symbol name
	pub fn generate_auto_symbol(&mut self, prefix: &str) -> String {
		self.auto_symbol_counter += 1;
		format!("{}_{}", prefix, self.auto_symbol_counter)
	}

	/// Enter a new scope
	pub fn enter_scope(&mut self, scope_type: ScopeType, name: Option<String>) -> usize {
		self.scope_manager.enter_scope(scope_type, name)
	}

	/// Exit current scope
	pub fn exit_scope(&mut self) -> AssemblyResult<()> {
		self.scope_manager
			.exit_scope()
			.map_err(|e| AssemblyError::internal(None, format!("Scope error: {}", e)))
	}

	/// Get symbol table
	pub fn symbol_table(&self) -> &SymbolTable {
		&self.table
	}

	/// Get mutable symbol table
	pub fn symbol_table_mut(&mut self) -> &mut SymbolTable {
		&mut self.table
	}

	/// Get scope manager
	pub fn scope_manager(&self) -> &ScopeManager {
		&self.scope_manager
	}

	/// Get symbol resolver
	pub fn resolver(&self) -> &SymbolResolver {
		&self.resolver
	}

	/// Resolve all forward references
	pub fn resolve_forward_references(&mut self) -> AssemblyResult<()> {
		self.resolver.resolve_all_forward_references(&mut self.table, &self.scope_manager)
	}

	/// Get undefined symbols
	pub fn undefined_symbols(&self) -> Vec<String> {
		self.resolver.undefined_symbols()
	}

	/// Export symbols to another symbol table
	pub fn export_symbols(&self, filter_visibility: SymbolVisibility) -> SymbolTable {
		let mut exported = SymbolTable::new();

		for (name, symbol) in self.table.symbols() {
			if symbol.visibility() == filter_visibility {
				exported.insert(name.clone(), symbol.clone()).ok();
			}
		}

		exported
	}

	/// Import symbols from another symbol table
	pub fn import_symbols(&mut self, other: &SymbolTable) -> AssemblyResult<()> {
		for (name, symbol) in other.symbols() {
			if symbol.visibility() == SymbolVisibility::Export {
				let mut imported_symbol = symbol.clone();
				imported_symbol.set_visibility(SymbolVisibility::Import);
				self.table.insert(name.clone(), imported_symbol)?;
			}
		}
		Ok(())
	}

	/// Generate symbol usage report
	pub fn usage_report(&self) -> SymbolUsageReport {
		let mut defined_count = 0;
		let mut undefined_count = 0;
		let mut referenced_count = 0;
		let mut unreferenced_count = 0;

		for symbol in self.table.symbols().values() {
			if symbol.is_defined() {
				defined_count += 1;
			} else {
				undefined_count += 1;
			}

			if symbol.is_referenced() {
				referenced_count += 1;
			} else {
				unreferenced_count += 1;
			}
		}

		SymbolUsageReport {
			total_symbols: self.table.len(),
			defined_count,
			undefined_count,
			referenced_count,
			unreferenced_count,
			forward_references: self.resolver.forward_reference_count(),
		}
	}
}

impl Default for SymbolManager {
	fn default() -> Self {
		Self::new()
	}
}

/// Symbol usage statistics
#[derive(Debug, Clone)]
pub struct SymbolUsageReport {
	pub total_symbols: usize,
	pub defined_count: usize,
	pub undefined_count: usize,
	pub referenced_count: usize,
	pub unreferenced_count: usize,
	pub forward_references: usize,
}

impl fmt::Display for SymbolUsageReport {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		writeln!(f, "Symbol Usage Report:")?;
		writeln!(f, "  Total symbols: {}", self.total_symbols)?;
		writeln!(f, "  Defined: {}", self.defined_count)?;
		writeln!(f, "  Undefined: {}", self.undefined_count)?;
		writeln!(f, "  Referenced: {}", self.referenced_count)?;
		writeln!(f, "  Unreferenced: {}", self.unreferenced_count)?;
		writeln!(f, "  Forward references: {}", self.forward_references)?;
		Ok(())
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
	fn test_symbol_visibility() {
		assert_eq!(format!("{}", SymbolVisibility::Local), "local");
		assert_eq!(format!("{}", SymbolVisibility::Global), "global");
		assert_eq!(format!("{}", SymbolVisibility::Export), "export");
		assert_eq!(format!("{}", SymbolVisibility::Import), "import");
	}

	#[test]
	fn test_symbol_status() {
		assert_eq!(format!("{}", SymbolStatus::Defined), "defined");
		assert_eq!(format!("{}", SymbolStatus::Undefined), "undefined");
	}

	#[test]
	fn test_symbol_attributes() {
		let mut attrs = SymbolAttributes::new();
		assert!(!attrs.is_constant);
		assert!(!attrs.is_referenced);

		attrs.mark_referenced();
		assert!(attrs.is_referenced);

		attrs.add_custom("test".to_string(), "value".to_string());
		assert_eq!(attrs.get_custom("test"), Some(&"value".to_string()));
	}

	#[test]
	fn test_cross_reference() {
		let pos = test_pos();
		let xref = CrossReference::from_pos(&pos, ReferenceType::Definition, "test".to_string());

		assert_eq!(xref.ref_type, ReferenceType::Definition);
		assert_eq!(xref.context, "test");
		assert_eq!(xref.line, 1);
		assert_eq!(xref.column, 1);
	}

	#[test]
	fn test_symbol_definition() {
		let pos = test_pos();
		let mut def = SymbolDefinition::new(
			"test".to_string(),
			SymbolType::Label,
			SymbolValue::Address(0x1000),
			pos,
			SymbolVisibility::Local,
		);

		assert!(def.is_defined());
		assert!(!def.is_constant());
		assert_eq!(def.reference_count(), 1); // Definition counts as one reference

		def.mark_referenced();
		assert!(def.attributes.is_referenced);
	}

	#[test]
	fn test_symbol_manager() {
		let mut manager = SymbolManager::new();
		let pos = test_pos();

		// Define a symbol
		assert!(
			manager
				.define_symbol(
					"test".to_string(),
					SymbolType::Label,
					SymbolValue::Address(0x1000),
					pos.clone(),
					SymbolVisibility::Local,
				)
				.is_ok()
		);

		// Resolve the symbol
		let value = manager.resolve_symbol("test", &pos).unwrap();
		assert_eq!(value, SymbolValue::Address(0x1000));

		// Generate auto symbol
		let auto_name = manager.generate_auto_symbol("temp");
		assert!(auto_name.starts_with("temp_"));
	}

	#[test]
	fn test_duplicate_symbol_definition() {
		let mut manager = SymbolManager::new();
		let pos = test_pos();

		// Define a symbol
		assert!(
			manager
				.define_symbol(
					"test".to_string(),
					SymbolType::Label,
					SymbolValue::Address(0x1000),
					pos.clone(),
					SymbolVisibility::Local,
				)
				.is_ok()
		);

		// Try to define it again (should fail)
		assert!(
			manager
				.define_symbol(
					"test".to_string(),
					SymbolType::Label,
					SymbolValue::Address(0x2000),
					pos,
					SymbolVisibility::Local,
				)
				.is_err()
		);
	}

	#[test]
	fn test_usage_report() {
		let mut manager = SymbolManager::new();
		let pos = test_pos();

		// Define some symbols
		manager
			.define_symbol(
				"symbol1".to_string(),
				SymbolType::Label,
				SymbolValue::Address(0x1000),
				pos.clone(),
				SymbolVisibility::Local,
			)
			.unwrap();

		manager
			.define_symbol(
				"symbol2".to_string(),
				SymbolType::Variable,
				SymbolValue::Number(42),
				pos.clone(),
				SymbolVisibility::Global,
			)
			.unwrap();

		// Reference one symbol
		manager.resolve_symbol("symbol1", &pos).unwrap();

		let report = manager.usage_report();
		assert_eq!(report.total_symbols, 2);
		assert_eq!(report.defined_count, 2);
		assert_eq!(report.undefined_count, 0);
		assert_eq!(report.referenced_count, 1);
		assert_eq!(report.unreferenced_count, 1);
	}
}
