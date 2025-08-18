//! User-defined functions for the NES compiler
//!
//! This module handles user-defined functions that can be used in expressions,
//! providing parameter substitution and evaluation functionality.

use crate::error::{AssemblyResult, MacroError, SourcePos};
use std::collections::HashMap;

/// A user-defined function definition
#[derive(Debug, Clone)]
pub struct FunctionDefinition {
	/// Function name
	name: String,
	/// Function body (expression)
	body: String,
	/// Number of parameters expected
	parameter_count: usize,
	/// Maximum parameter index found in the body
	max_parameter_index: usize,
}

impl FunctionDefinition {
	/// Create a new function definition
	pub fn new(name: String, body: String) -> Self {
		let mut function = Self {
			name,
			body,
			parameter_count: 0,
			max_parameter_index: 0,
		};

		function.analyze_parameters();
		function
	}

	/// Get the function name
	pub fn name(&self) -> &str {
		&self.name
	}

	/// Get the function body
	pub fn body(&self) -> &str {
		&self.body
	}

	/// Get the expected parameter count
	pub fn parameter_count(&self) -> usize {
		self.parameter_count
	}

	/// Get the maximum parameter index
	pub fn max_parameter_index(&self) -> usize {
		self.max_parameter_index
	}

	/// Evaluate the function with the given arguments
	pub fn evaluate(&self, args: &[i32]) -> AssemblyResult<i32> {
		// Validate argument count
		if args.len() != self.parameter_count {
			return Err(MacroError::ParameterError {
				pos: SourcePos::file_only(std::path::PathBuf::from("<function>")),
				message: format!(
					"Function '{}' expects {} arguments, got {}",
					self.name,
					self.parameter_count,
					args.len()
				),
			}
			.into());
		}

		// Substitute parameters in the function body
		let substituted_body = self.substitute_parameters(args)?;

		// Evaluate the resulting expression
		self.evaluate_expression(&substituted_body)
	}

	/// Analyze the function body to find parameter references
	fn analyze_parameters(&mut self) {
		let mut max_index = 0;

		for (i, ch) in self.body.chars().enumerate() {
			if ch == '\\' {
				if let Some(next_ch) = self.body.chars().nth(i + 1) {
					if next_ch.is_ascii_digit() {
						let param_index = next_ch.to_digit(10).unwrap() as usize;
						if param_index > 0 && param_index <= 9 {
							max_index = max_index.max(param_index);
						}
					}
				}
			}
		}

		self.max_parameter_index = max_index;
		self.parameter_count = max_index;
	}

	/// Substitute parameters in the function body
	fn substitute_parameters(&self, args: &[i32]) -> AssemblyResult<String> {
		let mut result = String::new();
		let mut chars = self.body.chars().peekable();

		while let Some(ch) = chars.next() {
			if ch == '\\' {
				if let Some(&next_ch) = chars.peek() {
					if next_ch.is_ascii_digit() {
						chars.next(); // consume the digit
						let param_index = next_ch.to_digit(10).unwrap() as usize;

						if param_index > 0 && param_index <= args.len() {
							result.push_str(&args[param_index - 1].to_string());
						} else {
							return Err(MacroError::ParameterError {
								pos: SourcePos::file_only(std::path::PathBuf::from("<function>")),
								message: format!("Invalid parameter index: \\{}", param_index),
							}
							.into());
						}
					} else {
						result.push(ch);
					}
				} else {
					result.push(ch);
				}
			} else {
				result.push(ch);
			}
		}

		Ok(result)
	}

	/// Evaluate a mathematical expression
	fn evaluate_expression(&self, expr: &str) -> AssemblyResult<i32> {
		// This is a simplified expression evaluator
		// In a real implementation, this would use a proper expression parser

		let trimmed = expr.trim();

		// Handle simple cases first
		if let Ok(value) = trimmed.parse::<i32>() {
			return Ok(value);
		}

		// Handle hexadecimal
		if trimmed.starts_with('$') {
			if let Ok(value) = i32::from_str_radix(&trimmed[1..], 16) {
				return Ok(value);
			}
		}

		// Handle binary
		if trimmed.starts_with('%') {
			if let Ok(value) = i32::from_str_radix(&trimmed[1..], 2) {
				return Ok(value);
			}
		}

		// For now, just handle simple arithmetic operations
		// A full implementation would use a proper expression parser
		if let Some(pos) = trimmed.find('+') {
			let left = self.evaluate_expression(&trimmed[..pos])?;
			let right = self.evaluate_expression(&trimmed[pos + 1..])?;
			return Ok(left + right);
		}

		if let Some(pos) = trimmed.find('-') {
			let left = self.evaluate_expression(&trimmed[..pos])?;
			let right = self.evaluate_expression(&trimmed[pos + 1..])?;
			return Ok(left - right);
		}

		if let Some(pos) = trimmed.find('*') {
			let left = self.evaluate_expression(&trimmed[..pos])?;
			let right = self.evaluate_expression(&trimmed[pos + 1..])?;
			return Ok(left * right);
		}

		if let Some(pos) = trimmed.find('/') {
			let left = self.evaluate_expression(&trimmed[..pos])?;
			let right = self.evaluate_expression(&trimmed[pos + 1..])?;
			if right == 0 {
				return Err(MacroError::ExpansionError {
					pos: SourcePos::file_only(std::path::PathBuf::from("<function>")),
					message: "Division by zero".to_string(),
				}
				.into());
			}
			return Ok(left / right);
		}

		Err(MacroError::ExpansionError {
			pos: SourcePos::file_only(std::path::PathBuf::from("<function>")),
			message: format!("Cannot evaluate expression: {}", expr),
		}
		.into())
	}

	/// Check if the function uses any parameters
	pub fn has_parameters(&self) -> bool {
		self.parameter_count > 0
	}

	/// Get a summary string for debugging
	pub fn summary(&self) -> String {
		format!(
			"Function '{}': {} parameters, body: '{}'",
			self.name, self.parameter_count, self.body
		)
	}
}

/// Manager for user-defined functions
#[derive(Debug)]
pub struct FunctionManager {
	/// Map of function name to definition
	functions: HashMap<String, FunctionDefinition>,
	/// Whether function names are case sensitive
	case_sensitive: bool,
}

impl FunctionManager {
	/// Create a new function manager
	pub fn new() -> Self {
		Self {
			functions: HashMap::new(),
			case_sensitive: true,
		}
	}

	/// Set case sensitivity for function names
	pub fn set_case_sensitive(&mut self, case_sensitive: bool) {
		self.case_sensitive = case_sensitive;
	}

	/// Normalize a function name according to case sensitivity
	fn normalize_name(&self, name: &str) -> String {
		if self.case_sensitive {
			name.to_string()
		} else {
			name.to_uppercase()
		}
	}

	/// Define a new function
	pub fn define_function(&mut self, name: String, body: String) -> AssemblyResult<()> {
		let normalized_name = self.normalize_name(&name);

		if self.functions.contains_key(&normalized_name) {
			return Err(MacroError::InvalidDefinition {
				pos: SourcePos::file_only(std::path::PathBuf::from("<function>")),
				message: format!("Function '{}' already defined", name),
			}
			.into());
		}

		let function_def = FunctionDefinition::new(normalized_name.clone(), body);
		self.functions.insert(normalized_name, function_def);

		Ok(())
	}

	/// Look up a function by name
	pub fn get_function(&self, name: &str) -> Option<&FunctionDefinition> {
		let normalized_name = self.normalize_name(name);
		self.functions.get(&normalized_name)
	}

	/// Check if a function is defined
	pub fn has_function(&self, name: &str) -> bool {
		let normalized_name = self.normalize_name(name);
		self.functions.contains_key(&normalized_name)
	}

	/// Call a function with the given arguments
	pub fn call_function(&self, name: &str, args: &[i32]) -> AssemblyResult<i32> {
		if let Some(function) = self.get_function(name) {
			function.evaluate(args)
		} else {
			Err(MacroError::NotFound {
				pos: SourcePos::file_only(std::path::PathBuf::from("<function>")),
				message: format!("Function '{}' not found", name),
			}
			.into())
		}
	}

	/// Get the number of defined functions
	pub fn function_count(&self) -> usize {
		self.functions.len()
	}

	/// Get all function names
	pub fn function_names(&self) -> Vec<String> {
		self.functions.keys().cloned().collect()
	}

	/// Get all functions
	pub fn functions(&self) -> &HashMap<String, FunctionDefinition> {
		&self.functions
	}

	/// Clear all functions
	pub fn clear(&mut self) {
		self.functions.clear();
	}

	/// Remove a specific function
	pub fn remove_function(&mut self, name: &str) -> bool {
		let normalized_name = self.normalize_name(name);
		self.functions.remove(&normalized_name).is_some()
	}

	/// Get functions that use a specific number of parameters
	pub fn functions_with_parameter_count(&self, count: usize) -> Vec<&FunctionDefinition> {
		self.functions.values().filter(|func| func.parameter_count() == count).collect()
	}
}

impl Default for FunctionManager {
	fn default() -> Self {
		Self::new()
	}
}

/// Built-in functions that are always available
pub struct BuiltinFunctions;

impl BuiltinFunctions {
	/// Get the names of all built-in functions
	pub fn function_names() -> &'static [&'static str] {
		&["LOW", "HIGH", "BANK", "PAGE", "SIZEOF"]
	}

	/// Evaluate a built-in function
	pub fn evaluate(name: &str, args: &[i32]) -> AssemblyResult<i32> {
		match name.to_uppercase().as_str() {
			"LOW" => {
				if args.len() != 1 {
					return Err(MacroError::ParameterError {
						pos: SourcePos::file_only(std::path::PathBuf::from("<builtin>")),
						message: "LOW function expects 1 argument".to_string(),
					}
					.into());
				}
				Ok(args[0] & 0xFF)
			}
			"HIGH" => {
				if args.len() != 1 {
					return Err(MacroError::ParameterError {
						pos: SourcePos::file_only(std::path::PathBuf::from("<builtin>")),
						message: "HIGH function expects 1 argument".to_string(),
					}
					.into());
				}
				Ok((args[0] >> 8) & 0xFF)
			}
			"BANK" => {
				if args.len() != 1 {
					return Err(MacroError::ParameterError {
						pos: SourcePos::file_only(std::path::PathBuf::from("<builtin>")),
						message: "BANK function expects 1 argument".to_string(),
					}
					.into());
				}
				// For now, just return a placeholder bank number
				// In a real implementation, this would look up the symbol's bank
				Ok((args[0] >> 13) & 0xFF)
			}
			"PAGE" => {
				if args.len() != 1 {
					return Err(MacroError::ParameterError {
						pos: SourcePos::file_only(std::path::PathBuf::from("<builtin>")),
						message: "PAGE function expects 1 argument".to_string(),
					}
					.into());
				}
				Ok((args[0] >> 8) & 0xFF)
			}
			"SIZEOF" => {
				if args.len() != 1 {
					return Err(MacroError::ParameterError {
						pos: SourcePos::file_only(std::path::PathBuf::from("<builtin>")),
						message: "SIZEOF function expects 1 argument".to_string(),
					}
					.into());
				}
				// For now, just return a placeholder size
				// In a real implementation, this would look up the symbol's size
				Ok(1)
			}
			_ => Err(MacroError::NotFound {
				pos: SourcePos::file_only(std::path::PathBuf::from("<builtin>")),
				message: format!("Builtin function '{}' not found", name),
			}
			.into()),
		}
	}

	/// Check if a function is a built-in function
	pub fn is_builtin(name: &str) -> bool {
		Self::function_names().contains(&name.to_uppercase().as_str())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_function_definition_creation() {
		let func = FunctionDefinition::new("test".to_string(), "\\1 + \\2".to_string());
		assert_eq!(func.name(), "test");
		assert_eq!(func.body(), "\\1 + \\2");
		assert_eq!(func.parameter_count(), 2);
		assert!(func.has_parameters());
	}

	#[test]
	fn test_function_evaluation() {
		let func = FunctionDefinition::new("add".to_string(), "\\1 + \\2".to_string());
		let result = func.evaluate(&[10, 20]).unwrap();
		assert_eq!(result, 30);
	}

	#[test]
	fn test_function_parameter_analysis() {
		let func = FunctionDefinition::new("test".to_string(), "\\1 * \\3 + \\2".to_string());
		assert_eq!(func.parameter_count(), 3);
		assert_eq!(func.max_parameter_index(), 3);
	}

	#[test]
	fn test_function_manager() {
		let mut manager = FunctionManager::new();
		assert_eq!(manager.function_count(), 0);

		manager.define_function("add".to_string(), "\\1 + \\2".to_string()).unwrap();
		assert_eq!(manager.function_count(), 1);
		assert!(manager.has_function("add"));

		let result = manager.call_function("add", &[5, 3]).unwrap();
		assert_eq!(result, 8);
	}

	#[test]
	fn test_function_redefinition() {
		let mut manager = FunctionManager::new();
		manager.define_function("test".to_string(), "\\1".to_string()).unwrap();
		assert!(manager.define_function("test".to_string(), "\\1 + 1".to_string()).is_err());
	}

	#[test]
	fn test_case_sensitivity() {
		let mut manager = FunctionManager::new();
		manager.set_case_sensitive(false);

		manager.define_function("Test".to_string(), "\\1".to_string()).unwrap();
		assert!(manager.has_function("TEST"));
		assert!(manager.has_function("test"));
		assert!(manager.has_function("Test"));
	}

	#[test]
	fn test_builtin_functions() {
		assert!(BuiltinFunctions::is_builtin("LOW"));
		assert!(BuiltinFunctions::is_builtin("HIGH"));
		assert!(!BuiltinFunctions::is_builtin("CUSTOM"));

		assert_eq!(BuiltinFunctions::evaluate("LOW", &[0x1234]).unwrap(), 0x34);
		assert_eq!(BuiltinFunctions::evaluate("HIGH", &[0x1234]).unwrap(), 0x12);
	}

	#[test]
	fn test_expression_evaluation() {
		let func = FunctionDefinition::new("complex".to_string(), "\\1 * 2 + \\2".to_string());
		let result = func.evaluate(&[10, 5]).unwrap();
		assert_eq!(result, 25); // 10 * 2 + 5 = 25
	}

	#[test]
	fn test_hex_and_binary_literals() {
		let func = FunctionDefinition::new("hex".to_string(), "$FF".to_string());
		let result = func.evaluate(&[]).unwrap();
		assert_eq!(result, 255);

		let func = FunctionDefinition::new("bin".to_string(), "%11111111".to_string());
		let result = func.evaluate(&[]).unwrap();
		assert_eq!(result, 255);
	}

	#[test]
	fn test_division_by_zero() {
		let func = FunctionDefinition::new("divide".to_string(), "\\1 / \\2".to_string());
		let result = func.evaluate(&[10, 0]);
		assert!(result.is_err());
	}

	#[test]
	fn test_wrong_parameter_count() {
		let func = FunctionDefinition::new("add".to_string(), "\\1 + \\2".to_string());
		let result = func.evaluate(&[10]); // Should need 2 parameters
		assert!(result.is_err());
	}
}
