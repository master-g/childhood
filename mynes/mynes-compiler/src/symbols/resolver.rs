//! Symbol resolution for the NES compiler
//!
//! This module handles the resolution of symbol references, including
//! forward references, expression evaluation, and cross-reference tracking.

use crate::error::{AssemblyResult, SourcePos, SymbolError};
use crate::symbols::{Symbol, SymbolLocation, SymbolTable, SymbolType, SymbolValue};
use std::collections::{HashMap, VecDeque};

/// Context for symbol resolution
#[derive(Debug)]
pub struct ResolutionContext {
	/// Current memory bank
	pub current_bank: usize,
	/// Current address within the bank
	pub current_address: u16,
	/// Current source line number
	pub current_line: usize,
	/// Whether we're in first pass (collecting symbols) or second pass (resolving)
	pub is_first_pass: bool,
}

impl ResolutionContext {
	/// Create a new resolution context
	pub fn new(bank: usize, address: u16, line: usize, is_first_pass: bool) -> Self {
		Self {
			current_bank: bank,
			current_address: address,
			current_line: line,
			is_first_pass,
		}
	}

	/// Get the current location
	pub fn current_location(&self) -> SymbolLocation {
		SourcePos::new(
			std::path::PathBuf::from(format!("bank_{}", self.current_bank)),
			self.current_line,
			1,
		)
	}
}

/// Symbol resolver that handles complex symbol resolution scenarios
#[derive(Debug)]
pub struct SymbolResolver {
	/// Pending forward references
	forward_references: VecDeque<PendingReference>,
	/// Symbol dependency graph for circular reference detection
	dependencies: HashMap<String, Vec<String>>,
	/// Resolution statistics
	stats: ResolutionStats,
}

/// A pending symbol reference that needs to be resolved
#[derive(Debug, Clone)]
pub struct PendingReference {
	/// The symbol being referenced
	pub symbol_name: String,
	/// Location where the reference occurs
	pub location: SymbolLocation,
	/// Type of reference
	pub reference_type: crate::symbols::ReferenceType,
	/// Context where this reference was created
	pub context: String,
	/// Whether this reference is required for first pass completion
	pub critical: bool,
}

impl PendingReference {
	/// Create a new pending reference
	pub fn new(
		symbol_name: String,
		location: SymbolLocation,
		reference_type: crate::symbols::ReferenceType,
		context: String,
	) -> Self {
		Self {
			symbol_name,
			location,
			reference_type,
			context,
			critical: false,
		}
	}

	/// Mark this reference as critical
	pub fn as_critical(mut self) -> Self {
		self.critical = true;
		self
	}
}

/// Statistics about symbol resolution
#[derive(Debug, Default, Clone)]
pub struct ResolutionStats {
	/// Number of symbols resolved
	pub symbols_resolved: usize,
	/// Number of forward references resolved
	pub forward_references_resolved: usize,
	/// Number of circular references detected
	pub circular_references: usize,
	/// Number of unresolved symbols
	pub unresolved_symbols: usize,
}

impl SymbolResolver {
	/// Create a new symbol resolver
	pub fn new() -> Self {
		Self {
			forward_references: VecDeque::new(),
			dependencies: HashMap::new(),
			stats: ResolutionStats::default(),
		}
	}

	/// Add a forward reference to be resolved later
	pub fn add_forward_reference(&mut self, symbol_name: String, pos: SourcePos) {
		let reference = PendingReference::new(
			symbol_name,
			pos,
			crate::symbols::ReferenceType::Usage,
			"forward reference".to_string(),
		);
		self.forward_references.push_back(reference);
	}

	/// Add a symbol dependency for circular reference detection
	pub fn add_dependency(&mut self, symbol: &str, depends_on: &str) {
		self.dependencies
			.entry(symbol.to_string())
			.or_insert_with(Vec::new)
			.push(depends_on.to_string());
	}

	/// Resolve all pending forward references
	pub fn resolve_forward_references(
		&mut self,
		symbol_table: &mut SymbolTable,
	) -> AssemblyResult<()> {
		let mut _resolved_count = 0;
		let mut remaining_references = VecDeque::new();

		// Try to resolve each forward reference
		while let Some(reference) = self.forward_references.pop_front() {
			match self.try_resolve_reference(&reference, symbol_table) {
				Ok(true) => {
					_resolved_count += 1;
					self.stats.forward_references_resolved += 1;
				}
				Ok(false) => {
					// Still unresolved, keep for next iteration
					remaining_references.push_back(reference);
				}
				Err(e) => {
					self.stats.unresolved_symbols += 1;
					return Err(e);
				}
			}
		}

		// Update the forward references queue
		self.forward_references = remaining_references;

		// Check for any remaining critical references
		for reference in &self.forward_references {
			if reference.critical {
				return Err(crate::error::AssemblyError::undefined_symbol(
					reference.location.clone(),
					reference.symbol_name.clone(),
				));
			}
		}

		Ok(())
	}

	/// Try to resolve a single reference
	fn try_resolve_reference(
		&self,
		reference: &PendingReference,
		symbol_table: &SymbolTable,
	) -> AssemblyResult<bool> {
		// Check if the symbol now exists
		if let Some(symbol) = symbol_table.get(&reference.symbol_name) {
			// Verify the symbol value is defined
			if symbol.value().is_defined() {
				self.validate_reference_type(symbol, reference)?;
				return Ok(true);
			}
		}

		// Symbol still not resolved
		Ok(false)
	}

	/// Get the count of pending forward references
	pub fn pending_reference_count(&self) -> usize {
		self.forward_references.len()
	}

	/// Get the count of forward references (alias for pending_reference_count)
	pub fn forward_reference_count(&self) -> usize {
		self.pending_reference_count()
	}

	/// Get list of undefined symbols
	pub fn undefined_symbols(&self) -> Vec<String> {
		self.forward_references.iter().map(|ref_| ref_.symbol_name.clone()).collect()
	}

	/// Resolve a symbol with context
	pub fn resolve(
		&self,
		name: &str,
		symbol_table: &SymbolTable,
		_scope_manager: &crate::symbols::scope::ScopeManager,
	) -> AssemblyResult<Option<crate::symbols::SymbolValue>> {
		if let Some(symbol) = symbol_table.get(name) {
			if symbol.value().is_defined() {
				Ok(Some(symbol.value().clone()))
			} else {
				Ok(None)
			}
		} else {
			Ok(None)
		}
	}

	/// Resolve all forward references with scope manager
	pub fn resolve_all_forward_references(
		&mut self,
		symbol_table: &mut SymbolTable,
		_scope_manager: &crate::symbols::scope::ScopeManager,
	) -> AssemblyResult<()> {
		self.resolve_forward_references(symbol_table)
	}

	/// Validate that a symbol can be used with the specified reference type
	fn validate_reference_type(
		&self,
		symbol: &Symbol,
		reference: &PendingReference,
	) -> AssemblyResult<()> {
		use crate::symbols::ReferenceType;

		match reference.reference_type {
			ReferenceType::Usage => {
				// Usage references are generally permissive
			}
			ReferenceType::Definition => {
				// Definition references are always valid
			}
			ReferenceType::Assignment => {
				// Assignment references are generally permissive
			}
			ReferenceType::Declaration => {
				// Declaration references are always valid
			}
		}

		Ok(())
	}

	/// Check for circular dependencies in symbol definitions
	pub fn check_circular_dependencies(&mut self) -> AssemblyResult<()> {
		for symbol in self.dependencies.keys() {
			if self.has_circular_dependency(symbol, &mut Vec::new())? {
				self.stats.circular_references += 1;
				return Err(crate::error::AssemblyError::symbol(
					SourcePos::file_only(std::path::PathBuf::from("resolver")),
					format!("Circular dependency detected involving symbol: {}", symbol),
				));
			}
		}

		Ok(())
	}

	/// Recursively check if a symbol has circular dependencies
	fn has_circular_dependency(
		&self,
		symbol: &str,
		visited: &mut Vec<String>,
	) -> AssemblyResult<bool> {
		if visited.contains(&symbol.to_string()) {
			return Ok(true);
		}

		visited.push(symbol.to_string());

		if let Some(dependencies) = self.dependencies.get(symbol) {
			for dep in dependencies {
				if self.has_circular_dependency(dep, visited)? {
					return Ok(true);
				}
			}
		}

		visited.pop();
		Ok(false)
	}

	/// Resolve a symbol value in the context of an expression
	pub fn resolve_symbol_value(
		&mut self,
		name: &str,
		symbol_table: &mut SymbolTable,
		context: &ResolutionContext,
	) -> AssemblyResult<Option<i32>> {
		if let Some(symbol) = symbol_table.get(name) {
			match &symbol.value() {
				SymbolValue::Number(val) => {
					self.stats.symbols_resolved += 1;
					Ok(Some(*val))
				}
				SymbolValue::Address(addr) => {
					self.stats.symbols_resolved += 1;
					Ok(Some(*addr as i32))
				}
				SymbolValue::Undefined => {
					// Add as forward reference if in first pass
					if context.is_first_pass {
						self.add_forward_reference(name.to_string(), context.current_location());
						Ok(None)
					} else {
						Err(crate::error::AssemblyError::undefined_symbol(
							context.current_location(),
							name.to_string(),
						))
					}
				}
				SymbolValue::String(_) => Err(crate::error::AssemblyError::symbol(
					context.current_location(),
					"Cannot use string symbol in numeric context".to_string(),
				)),
				SymbolValue::Expression(_) => {
					// For now, treat expressions as unresolved
					if context.is_first_pass {
						self.add_forward_reference(name.to_string(), context.current_location());
						Ok(None)
					} else {
						Err(crate::error::AssemblyError::undefined_symbol(
							context.current_location(),
							name.to_string(),
						))
					}
				}
			}
		} else {
			// Symbol not found
			if context.is_first_pass {
				// Add as forward reference
				self.add_forward_reference(name.to_string(), context.current_location());
				Ok(None)
			} else {
				Err(crate::error::AssemblyError::undefined_symbol(
					context.current_location(),
					name.to_string(),
				))
			}
		}
	}

	/// Get the count of critical pending references
	pub fn critical_reference_count(&self) -> usize {
		self.forward_references.iter().filter(|r| r.critical).count()
	}

	/// Get resolution statistics
	pub fn stats(&self) -> &ResolutionStats {
		&self.stats
	}

	/// Clear all pending references and dependencies
	pub fn clear(&mut self) {
		self.forward_references.clear();
		self.dependencies.clear();
		self.stats = ResolutionStats::default();
	}

	/// Get all pending references for a specific symbol
	pub fn get_references_for_symbol(&self, symbol_name: &str) -> Vec<&PendingReference> {
		self.forward_references.iter().filter(|r| r.symbol_name == symbol_name).collect()
	}

	/// Remove all references to a specific symbol
	pub fn remove_references_for_symbol(&mut self, symbol_name: &str) {
		self.forward_references.retain(|r| r.symbol_name != symbol_name);
	}
}

impl Default for SymbolResolver {
	fn default() -> Self {
		Self::new()
	}
}

/// Forward reference type for compatibility
pub type ForwardReference = PendingReference;

#[cfg(test)]
mod tests {
	use super::*;
	use std::path::PathBuf;

	fn test_pos() -> SourcePos {
		SourcePos::new(PathBuf::from("test.asm"), 1, 1)
	}

	#[test]
	fn test_resolver_creation() {
		let resolver = SymbolResolver::new();
		assert_eq!(resolver.pending_reference_count(), 0);
		assert_eq!(resolver.critical_reference_count(), 0);
	}

	#[test]
	fn test_add_forward_reference() {
		let mut resolver = SymbolResolver::new();
		let pos = test_pos();

		resolver.add_forward_reference("test_symbol".to_string(), pos);
		assert_eq!(resolver.pending_reference_count(), 1);
	}

	#[test]
	fn test_resolution_context() {
		let context = ResolutionContext::new(1, 0x8000, 42, true);
		assert_eq!(context.current_bank, 1);
		assert_eq!(context.current_address, 0x8000);
		assert_eq!(context.current_line, 42);
		assert!(context.is_first_pass);
	}
}
