//! Scope management for symbol resolution
//!
//! This module handles scoping rules for symbols, including global and local
//! symbol visibility and nested scope management.

use crate::error::{AssemblyResult, SymbolError};
use crate::symbols::Symbol;
use std::collections::HashMap;

/// Type alias for scope compatibility
pub type SymbolScope = Scope;

/// Types of scopes in the assembler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeType {
	/// Global scope - symbols visible everywhere
	Global,
	/// Local scope - symbols visible only within current context
	Local,
	/// Macro scope - symbols visible only within macro expansion
	Macro,
	/// Procedure scope - symbols visible only within procedure
	Procedure,
}

/// A scope context that manages symbol visibility
#[derive(Debug, Clone)]
pub struct Scope {
	/// Type of this scope
	scope_type: ScopeType,
	/// Name of this scope (for debugging)
	name: String,
	/// Symbols defined in this scope
	symbols: HashMap<String, Symbol>,
	/// Parent scope (if any)
	parent: Option<Box<Scope>>,
	/// Unique identifier for this scope
	id: usize,
}

impl Scope {
	/// Create a new scope
	pub fn new(scope_type: ScopeType, name: String, id: usize) -> Self {
		Self {
			scope_type,
			name,
			symbols: HashMap::new(),
			parent: None,
			id,
		}
	}

	/// Create a new global scope
	pub fn global() -> Self {
		Self::new(ScopeType::Global, "global".to_string(), 0)
	}

	/// Create a new local scope with a parent
	pub fn local(name: String, parent: Scope, id: usize) -> Self {
		let mut scope = Self::new(ScopeType::Local, name, id);
		scope.parent = Some(Box::new(parent));
		scope
	}

	/// Create a new macro scope with a parent
	pub fn macro_scope(name: String, parent: Scope, id: usize) -> Self {
		let mut scope = Self::new(ScopeType::Macro, name, id);
		scope.parent = Some(Box::new(parent));
		scope
	}

	/// Create a new procedure scope with a parent
	pub fn procedure(name: String, parent: Scope, id: usize) -> Self {
		let mut scope = Self::new(ScopeType::Procedure, name, id);
		scope.parent = Some(Box::new(parent));
		scope
	}

	/// Get the scope type
	pub fn scope_type(&self) -> ScopeType {
		self.scope_type
	}

	/// Get the scope name
	pub fn name(&self) -> &str {
		&self.name
	}

	/// Get the scope ID
	pub fn id(&self) -> usize {
		self.id
	}

	/// Check if this scope has a parent
	pub fn has_parent(&self) -> bool {
		self.parent.is_some()
	}

	/// Get a reference to the parent scope
	pub fn parent(&self) -> Option<&Scope> {
		self.parent.as_ref().map(|p| p.as_ref())
	}

	/// Define a symbol in this scope
	pub fn define_symbol(&mut self, name: String, symbol: Symbol) -> AssemblyResult<()> {
		if self.symbols.contains_key(&name) {
			return Err(SymbolError::AlreadyExists {
				name,
			}
			.into());
		}

		self.symbols.insert(name, symbol);
		Ok(())
	}

	/// Look up a symbol in this scope only
	pub fn lookup_local(&self, name: &str) -> Option<&Symbol> {
		self.symbols.get(name)
	}

	/// Look up a symbol in this scope and parent scopes
	pub fn lookup(&self, name: &str) -> Option<&Symbol> {
		// First check local scope
		if let Some(symbol) = self.symbols.get(name) {
			return Some(symbol);
		}

		// Then check parent scope if it exists
		if let Some(ref parent) = self.parent {
			return parent.lookup(name);
		}

		None
	}

	/// Check if a symbol exists in this scope only
	pub fn contains_local(&self, name: &str) -> bool {
		self.symbols.contains_key(name)
	}

	/// Check if a symbol exists in this scope or parent scopes
	pub fn contains(&self, name: &str) -> bool {
		self.lookup(name).is_some()
	}

	/// Get all symbols in this scope
	pub fn symbols(&self) -> &HashMap<String, Symbol> {
		&self.symbols
	}

	/// Get all symbols in this scope (mutable)
	pub fn symbols_mut(&mut self) -> &mut HashMap<String, Symbol> {
		&mut self.symbols
	}

	/// Get count of symbols in this scope only
	pub fn symbol_count(&self) -> usize {
		self.symbols.len()
	}

	/// Get count of symbols in this scope and all parent scopes
	pub fn total_symbol_count(&self) -> usize {
		let mut count = self.symbols.len();
		if let Some(ref parent) = self.parent {
			count += parent.total_symbol_count();
		}
		count
	}

	/// Clear all symbols from this scope
	pub fn clear(&mut self) {
		self.symbols.clear();
	}

	/// Get the depth of this scope (0 for global, 1 for first level, etc.)
	pub fn depth(&self) -> usize {
		if let Some(ref parent) = self.parent {
			parent.depth() + 1
		} else {
			0
		}
	}

	/// Check if this scope is a global scope
	pub fn is_global(&self) -> bool {
		self.scope_type == ScopeType::Global
	}

	/// Check if this scope is a local scope
	pub fn is_local(&self) -> bool {
		self.scope_type == ScopeType::Local
	}

	/// Check if this scope is a macro scope
	pub fn is_macro(&self) -> bool {
		self.scope_type == ScopeType::Macro
	}

	/// Check if this scope is a procedure scope
	pub fn is_procedure(&self) -> bool {
		self.scope_type == ScopeType::Procedure
	}

	/// Find the nearest parent scope of a specific type
	pub fn find_parent_of_type(&self, scope_type: ScopeType) -> Option<&Scope> {
		if self.scope_type == scope_type {
			return Some(self);
		}

		if let Some(ref parent) = self.parent {
			return parent.find_parent_of_type(scope_type);
		}

		None
	}

	/// Get the full scope path (for debugging)
	pub fn full_path(&self) -> String {
		if let Some(ref parent) = self.parent {
			format!("{}.{}", parent.full_path(), self.name)
		} else {
			self.name.clone()
		}
	}
}

/// Scope manager that maintains a stack of scopes
#[derive(Debug)]
pub struct ScopeManager {
	/// Current scope stack
	scope_stack: Vec<Scope>,
	/// Next scope ID to assign
	next_id: usize,
	/// Current label for local label context
	current_label: Option<String>,
}

impl ScopeManager {
	/// Create a new scope manager with a global scope
	pub fn new() -> Self {
		let mut manager = Self {
			scope_stack: Vec::new(),
			next_id: 1,
			current_label: None,
		};

		// Start with global scope
		manager.scope_stack.push(Scope::global());
		manager
	}

	/// Get the current scope
	pub fn current_scope(&self) -> &Scope {
		self.scope_stack.last().expect("Scope stack should never be empty")
	}

	/// Get the current scope (mutable)
	pub fn current_scope_mut(&mut self) -> &mut Scope {
		self.scope_stack.last_mut().expect("Scope stack should never be empty")
	}

	/// Push a new scope onto the stack
	pub fn push_scope(&mut self, scope_type: ScopeType, name: String) {
		let id = self.next_id;
		self.next_id += 1;

		let current = self.scope_stack.pop().expect("Scope stack should never be empty");
		let new_scope = match scope_type {
			ScopeType::Global => Scope::global(),
			ScopeType::Local => Scope::local(name, current, id),
			ScopeType::Macro => Scope::macro_scope(name, current, id),
			ScopeType::Procedure => Scope::procedure(name, current, id),
		};

		self.scope_stack.push(new_scope);
	}

	/// Pop the current scope from the stack
	pub fn pop_scope(&mut self) -> Result<Scope, SymbolError> {
		if self.scope_stack.len() <= 1 {
			return Err(SymbolError::ScopeError {
				message: "Cannot pop global scope".to_string(),
			});
		}

		let current = self.scope_stack.pop().unwrap();

		// Restore parent scope
		if let Some(parent) = current.parent.clone() {
			self.scope_stack.push(*parent);
		}

		Ok(current)
	}

	/// Get the depth of the current scope stack
	pub fn depth(&self) -> usize {
		self.scope_stack.len()
	}

	/// Get the current scope ID
	pub fn current_scope_id(&self) -> usize {
		self.current_scope().id
	}

	/// Enter a new scope
	pub fn enter_scope(&mut self, scope_type: ScopeType, name: Option<String>) -> usize {
		let scope_name = name.unwrap_or_else(|| format!("scope_{}", self.next_id));
		self.push_scope(scope_type, scope_name);
		self.current_scope().id
	}

	/// Exit current scope
	pub fn exit_scope(&mut self) -> Result<(), crate::error::AssemblyError> {
		self.pop_scope().map(|_| ()).map_err(|e| crate::error::AssemblyError::Internal {
			pos: Some(crate::error::SourcePos::new("".into(), 0, 0)),
			message: format!("Scope error: {:?}", e),
		})
	}

	/// Check if we're in the global scope
	pub fn is_global(&self) -> bool {
		self.scope_stack.len() == 1
	}

	/// Define a symbol in the current scope
	pub fn define_symbol(&mut self, name: String, symbol: Symbol) -> AssemblyResult<()> {
		self.current_scope_mut().define_symbol(name, symbol)
	}

	/// Look up a symbol starting from the current scope
	pub fn lookup_symbol(&self, name: &str) -> Option<&Symbol> {
		self.current_scope().lookup(name)
	}

	/// Check if a symbol exists in the current scope chain
	pub fn contains_symbol(&self, name: &str) -> bool {
		self.current_scope().contains(name)
	}

	/// Get all scopes in the current stack
	pub fn scopes(&self) -> &[Scope] {
		&self.scope_stack
	}

	/// Find a scope by ID
	pub fn find_scope_by_id(&self, id: usize) -> Option<&Scope> {
		for scope in &self.scope_stack {
			if scope.id() == id {
				return Some(scope);
			}
			// TODO: Search in parent scopes if needed
		}
		None
	}

	/// Get the current scope path
	pub fn current_path(&self) -> String {
		self.current_scope().full_path()
	}

	/// Set the current label for local label context
	pub fn set_current_label(&mut self, label: Option<String>) {
		self.current_label = label;
	}

	/// Get the current label
	pub fn current_label(&self) -> Option<&String> {
		self.current_label.as_ref()
	}
}

impl Default for ScopeManager {
	fn default() -> Self {
		Self::new()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::error::SourcePos;
	use crate::symbols::{SymbolInfo, SymbolType, SymbolValue};

	fn create_test_symbol(name: &str, value: i32) -> SymbolInfo {
		SymbolInfo::new(
			SymbolType::Constant,
			SymbolValue::Number(value),
			SourcePos::new("test.s".into(), 1, 1),
			0,
		)
	}

	#[test]
	fn test_scope_creation() {
		let scope = Scope::global();
		assert_eq!(scope.scope_type(), ScopeType::Global);
		assert_eq!(scope.name(), "global");
		assert!(!scope.has_parent());
		assert_eq!(scope.depth(), 0);
	}

	#[test]
	fn test_nested_scope() {
		let global = Scope::global();
		let local = Scope::local("test".to_string(), global, 1);

		assert_eq!(local.scope_type(), ScopeType::Local);
		assert_eq!(local.name(), "test");
		assert!(local.has_parent());
		assert_eq!(local.depth(), 1);
	}

	#[test]
	fn test_symbol_definition() {
		let mut scope = Scope::global();
		let symbol = create_test_symbol("TEST", 42);

		assert!(scope.define_symbol("TEST".to_string(), symbol).is_ok());
		assert!(scope.contains_local("TEST"));
		assert_eq!(scope.symbol_count(), 1);
	}

	#[test]
	fn test_symbol_lookup() {
		let mut global = Scope::global();
		let symbol1 = create_test_symbol("GLOBAL", 1);
		global.define_symbol("GLOBAL".to_string(), symbol1).unwrap();

		let mut local = Scope::local("test".to_string(), global, 1);
		let symbol2 = create_test_symbol("LOCAL", 2);
		local.define_symbol("LOCAL".to_string(), symbol2).unwrap();

		// Should find local symbol
		assert!(local.lookup("LOCAL").is_some());
		// Should find global symbol through parent
		assert!(local.lookup("GLOBAL").is_some());
		// Should not find non-existent symbol
		assert!(local.lookup("MISSING").is_none());
	}

	#[test]
	fn test_scope_manager() {
		let mut manager = ScopeManager::new();
		assert!(manager.is_global());
		assert_eq!(manager.depth(), 1);

		manager.push_scope(ScopeType::Local, "test".to_string());
		assert!(!manager.is_global());
		assert_eq!(manager.depth(), 1); // Still 1 because we replace the current scope

		let symbol = create_test_symbol("TEST", 42);
		assert!(manager.define_symbol("TEST".to_string(), symbol).is_ok());
		assert!(manager.contains_symbol("TEST"));
	}

	#[test]
	fn test_scope_manager_pop() {
		let mut manager = ScopeManager::new();
		manager.push_scope(ScopeType::Local, "test".to_string());

		let symbol = create_test_symbol("LOCAL", 1);
		manager.define_symbol("LOCAL".to_string(), symbol).unwrap();

		// Pop the local scope
		let popped = manager.pop_scope().unwrap();
		assert_eq!(popped.name(), "test");

		// Should not find local symbol anymore
		assert!(!manager.contains_symbol("LOCAL"));
	}

	#[test]
	fn test_scope_types() {
		let global = Scope::global();
		assert!(global.is_global());
		assert!(!global.is_local());

		let local = Scope::local("test".to_string(), global, 1);
		assert!(!local.is_global());
		assert!(local.is_local());
		assert!(!local.is_macro());
		assert!(!local.is_procedure());
	}

	#[test]
	fn test_full_path() {
		let global = Scope::global();
		let local1 = Scope::local("level1".to_string(), global, 1);
		let local2 = Scope::local("level2".to_string(), local1, 2);

		assert_eq!(local2.full_path(), "global.level1.level2");
	}

	#[test]
	fn test_find_parent_of_type() {
		let global = Scope::global();
		let proc = Scope::procedure("proc1".to_string(), global, 1);
		let local = Scope::local("local1".to_string(), proc, 2);

		let found_proc = local.find_parent_of_type(ScopeType::Procedure);
		assert!(found_proc.is_some());
		assert_eq!(found_proc.unwrap().name(), "proc1");

		let found_global = local.find_parent_of_type(ScopeType::Global);
		assert!(found_global.is_some());
		assert_eq!(found_global.unwrap().name(), "global");
	}
}
