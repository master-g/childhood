//! Macro system for the NES compiler
//!
//! This module provides macro definition, expansion, and management functionality
//! for the assembler, including parameter substitution and nested macro support.

pub mod definition;
pub mod expansion;
pub mod functions;

pub use definition::{MacroDefinition, MacroParameter};
pub use expansion::{MacroExpander, MacroExpansionContext};
pub use functions::{FunctionDefinition, FunctionManager};

use crate::error::{AssemblyResult, MacroError, SourcePos};
use std::collections::HashMap;

/// Main macro manager that coordinates macro operations
#[derive(Debug)]
pub struct MacroManager {
	/// Defined macros
	macros: HashMap<String, MacroDefinition>,
	/// User-defined functions
	functions: FunctionManager,
	/// Macro expansion context stack
	expansion_stack: Vec<MacroExpansionContext>,
	/// Maximum expansion depth to prevent infinite recursion
	max_expansion_depth: usize,
}

impl MacroManager {
	/// Create a new macro manager
	pub fn new() -> Self {
		Self {
			macros: HashMap::new(),
			functions: FunctionManager::new(),
			expansion_stack: Vec::new(),
			max_expansion_depth: 32,
		}
	}

	/// Define a new macro
	pub fn define_macro(&mut self, macro_def: MacroDefinition) -> AssemblyResult<()> {
		let name = macro_def.name().to_string();

		if self.macros.contains_key(&name) {
			return Err(MacroError::InvalidDefinition {
				pos: SourcePos::file_only(std::path::PathBuf::from("<macro>")),
				message: format!("Macro '{}' already defined", name),
			}
			.into());
		}

		self.macros.insert(name, macro_def);
		Ok(())
	}

	/// Expand a macro with the given arguments
	pub fn expand_macro(&mut self, name: &str, args: &[String]) -> AssemblyResult<Vec<String>> {
		// Check expansion depth
		if self.expansion_stack.len() >= self.max_expansion_depth {
			return Err(MacroError::ExpansionError {
				pos: SourcePos::file_only(std::path::PathBuf::from("<macro>")),
				message: "Maximum macro expansion depth exceeded".to_string(),
			}
			.into());
		}

		// Look up the macro
		if let Some(macro_def) = self.macros.get(name) {
			// Create expansion context
			let context = MacroExpansionContext::new(name.to_string(), args.to_vec());
			self.expansion_stack.push(context);

			// Perform expansion
			let result = macro_def.expand(args);

			// Pop context
			self.expansion_stack.pop();

			result
		} else {
			Err(MacroError::NotFound {
				pos: SourcePos::file_only(std::path::PathBuf::from("<macro>")),
				message: format!("Macro '{}' not found", name),
			}
			.into())
		}
	}

	/// Check if a macro is defined
	pub fn has_macro(&self, name: &str) -> bool {
		self.macros.contains_key(name)
	}

	/// Get the number of defined macros
	pub fn macro_count(&self) -> usize {
		self.macros.len()
	}

	/// Get all macro names
	pub fn macro_names(&self) -> Vec<String> {
		self.macros.keys().cloned().collect()
	}

	/// Set maximum expansion depth
	pub fn set_max_expansion_depth(&mut self, depth: usize) {
		self.max_expansion_depth = depth;
	}

	/// Get current expansion depth
	pub fn current_expansion_depth(&self) -> usize {
		self.expansion_stack.len()
	}

	/// Check if currently expanding a macro
	pub fn is_expanding(&self) -> bool {
		!self.expansion_stack.is_empty()
	}

	/// Get the current expansion context
	pub fn current_expansion(&self) -> Option<&MacroExpansionContext> {
		self.expansion_stack.last()
	}

	/// Clear all macros and functions
	pub fn clear(&mut self) {
		self.macros.clear();
		self.functions.clear();
		self.expansion_stack.clear();
	}

	/// Get a reference to the function manager
	pub fn functions(&self) -> &FunctionManager {
		&self.functions
	}

	/// Get a mutable reference to the function manager
	pub fn functions_mut(&mut self) -> &mut FunctionManager {
		&mut self.functions
	}
}

impl Default for MacroManager {
	fn default() -> Self {
		Self::new()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_macro_manager_creation() {
		let manager = MacroManager::new();
		assert_eq!(manager.macro_count(), 0);
		assert!(!manager.is_expanding());
		assert_eq!(manager.current_expansion_depth(), 0);
	}

	// Additional tests will be added once the other modules are implemented
}
