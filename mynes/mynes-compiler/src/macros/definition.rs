//! Macro definition structures for the NES compiler
//!
//! This module defines the data structures and functionality for macro definitions,
//! including parameter handling and macro body storage.

use crate::error::{AssemblyResult, MacroError, SourcePos};

/// A macro parameter definition
#[derive(Debug, Clone)]
pub struct MacroParameter {
	/// Parameter name (e.g., "\1", "\2", etc.)
	pub name: String,
	/// Parameter index (1-based)
	pub index: usize,
	/// Whether this parameter is optional
	pub optional: bool,
}

impl MacroParameter {
	/// Create a new macro parameter
	pub fn new(index: usize) -> Self {
		Self {
			name: format!("\\{}", index),
			index,
			optional: false,
		}
	}

	/// Create an optional macro parameter
	pub fn optional(index: usize) -> Self {
		Self {
			name: format!("\\{}", index),
			index,
			optional: true,
		}
	}
}

/// A complete macro definition
#[derive(Debug, Clone)]
pub struct MacroDefinition {
	/// Macro name
	name: String,
	/// Macro parameters
	parameters: Vec<MacroParameter>,
	/// Macro body (lines of assembly code)
	body: Vec<String>,
	/// Maximum parameter index used in the macro
	max_param_index: usize,
	/// Unique counter for local labels
	local_counter: u32,
}

impl MacroDefinition {
	/// Create a new macro definition
	pub fn new(name: String, body: Vec<String>) -> Self {
		let mut definition = Self {
			name,
			parameters: Vec::new(),
			body,
			max_param_index: 0,
			local_counter: 0,
		};

		definition.analyze_parameters();
		definition
	}

	/// Get the macro name
	pub fn name(&self) -> &str {
		&self.name
	}

	/// Get the macro body
	pub fn body(&self) -> &[String] {
		&self.body
	}

	/// Get the macro parameters
	pub fn parameters(&self) -> &[MacroParameter] {
		&self.parameters
	}

	/// Get the maximum parameter index
	pub fn max_param_index(&self) -> usize {
		self.max_param_index
	}

	/// Expand the macro with the given arguments
	pub fn expand(&self, args: &[String]) -> AssemblyResult<Vec<String>> {
		// Validate argument count
		if args.len() > self.max_param_index {
			return Err(MacroError::ParameterError {
				pos: SourcePos::file_only(std::path::PathBuf::from("<macro>")),
				message: format!(
					"Too many arguments: expected at most {}, got {}",
					self.max_param_index,
					args.len()
				),
			}
			.into());
		}

		let mut expanded_lines = Vec::new();
		let local_id = self.generate_local_id();

		for line in &self.body {
			let expanded_line = self.expand_line(line, args, local_id)?;
			expanded_lines.push(expanded_line);
		}

		Ok(expanded_lines)
	}

	/// Analyze the macro body to find parameter references
	fn analyze_parameters(&mut self) {
		let mut max_index = 0;
		let mut found_params = Vec::new();

		for line in &self.body {
			for (i, ch) in line.chars().enumerate() {
				if ch == '\\' {
					if let Some(next_ch) = line.chars().nth(i + 1) {
						if next_ch.is_ascii_digit() {
							let param_index = next_ch.to_digit(10).unwrap() as usize;
							if param_index > 0 && param_index <= 9 {
								max_index = max_index.max(param_index);
								if !found_params.contains(&param_index) {
									found_params.push(param_index);
								}
							}
						}
					}
				}
			}
		}

		self.max_param_index = max_index;
		self.parameters = found_params.into_iter().map(MacroParameter::new).collect();
	}

	/// Expand a single line with parameter substitution
	fn expand_line(&self, line: &str, args: &[String], local_id: u32) -> AssemblyResult<String> {
		let mut result = String::new();
		let mut chars = line.chars().peekable();

		while let Some(ch) = chars.next() {
			if ch == '\\' {
				if let Some(&next_ch) = chars.peek() {
					match next_ch {
						// Parameter substitution \1-\9
						'1'..='9' => {
							chars.next(); // consume the digit
							let param_index = next_ch.to_digit(10).unwrap() as usize;

							if param_index <= args.len() {
								result.push_str(&args[param_index - 1]);
							} else {
								// Parameter not provided, leave empty or use default
								// For now, just leave empty
							}
						}
						// Parameter count \#
						'#' => {
							chars.next(); // consume the #
							result.push_str(&args.len().to_string());
						}
						// Local label unique ID \@
						'@' => {
							chars.next(); // consume the @
							result.push_str(&local_id.to_string());
						}
						// Parameter type queries \?1-\?9
						'?' => {
							chars.next(); // consume the ?
							if let Some(&digit_ch) = chars.peek() {
								if digit_ch.is_ascii_digit() {
									chars.next(); // consume the digit
									let param_index = digit_ch.to_digit(10).unwrap() as usize;

									if param_index <= args.len() && param_index > 0 {
										let arg_type =
											self.determine_argument_type(&args[param_index - 1]);
										result.push_str(&arg_type.to_string());
									} else {
										result.push('0'); // ARG_NONE
									}
								} else {
									result.push('\\');
									result.push('?');
								}
							} else {
								result.push('\\');
								result.push('?');
							}
						}
						// Literal backslash \\
						'\\' => {
							chars.next(); // consume the second backslash
							result.push('\\');
						}
						_ => {
							// Not a recognized escape sequence, keep the backslash
							result.push(ch);
						}
					}
				} else {
					// Backslash at end of line
					result.push(ch);
				}
			} else {
				result.push(ch);
			}
		}

		Ok(result)
	}

	/// Determine the type of a macro argument
	fn determine_argument_type(&self, arg: &str) -> u8 {
		let trimmed = arg.trim();

		if trimmed.is_empty() {
			return 0; // ARG_NONE
		}

		// Check for register names
		if matches!(trimmed.to_uppercase().as_str(), "A" | "X" | "Y") {
			return 1; // ARG_REG
		}

		// Check for immediate addressing
		if trimmed.starts_with('#') {
			return 2; // ARG_IMMEDIATE
		}

		// Check for indirect addressing
		if trimmed.starts_with('[') && trimmed.ends_with(']') {
			return 4; // ARG_INDIRECT
		}

		// Check for string literal
		if (trimmed.starts_with('"') && trimmed.ends_with('"'))
			|| (trimmed.starts_with('\'') && trimmed.ends_with('\''))
		{
			return 5; // ARG_STRING
		}

		// Check if it looks like a label/symbol
		if trimmed.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.') {
			return 6; // ARG_LABEL
		}

		// Default to absolute addressing
		3 // ARG_ABSOLUTE
	}

	/// Generate a unique local label identifier
	fn generate_local_id(&self) -> u32 {
		// In a real implementation, this would be managed globally
		// For now, just use a simple counter
		use std::sync::atomic::{AtomicU32, Ordering};
		static COUNTER: AtomicU32 = AtomicU32::new(1);
		COUNTER.fetch_add(1, Ordering::Relaxed)
	}

	/// Get the number of lines in the macro body
	pub fn line_count(&self) -> usize {
		self.body.len()
	}

	/// Check if the macro uses any parameters
	pub fn has_parameters(&self) -> bool {
		self.max_param_index > 0
	}

	/// Check if the macro uses local labels
	pub fn uses_local_labels(&self) -> bool {
		self.body.iter().any(|line| line.contains("\\@"))
	}

	/// Get a summary string for debugging
	pub fn summary(&self) -> String {
		format!(
			"Macro '{}': {} lines, {} parameters, uses local labels: {}",
			self.name,
			self.body.len(),
			self.max_param_index,
			self.uses_local_labels()
		)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_macro_definition_creation() {
		let body = vec!["LDA \\1".to_string(), "STA \\2".to_string()];
		let macro_def = MacroDefinition::new("test_macro".to_string(), body);

		assert_eq!(macro_def.name(), "test_macro");
		assert_eq!(macro_def.line_count(), 2);
		assert_eq!(macro_def.max_param_index(), 2);
		assert!(macro_def.has_parameters());
	}

	#[test]
	fn test_parameter_analysis() {
		let body = vec!["LDA \\1".to_string(), "STA \\3".to_string(), "INX".to_string()];
		let macro_def = MacroDefinition::new("test".to_string(), body);

		assert_eq!(macro_def.max_param_index(), 3);
		assert_eq!(macro_def.parameters().len(), 2); // \1 and \3
	}

	#[test]
	fn test_macro_expansion() {
		let body = vec!["LDA \\1".to_string(), "STA \\2".to_string()];
		let macro_def = MacroDefinition::new("test".to_string(), body);

		let args = vec!["#$42".to_string(), "$0200".to_string()];
		let expanded = macro_def.expand(&args).unwrap();

		assert_eq!(expanded.len(), 2);
		assert_eq!(expanded[0], "LDA #$42");
		assert_eq!(expanded[1], "STA $0200");
	}

	#[test]
	fn test_special_parameters() {
		let body = vec![
			"arg_count: \\#".to_string(),
			"local_label\\@:".to_string(),
			"arg1_type: \\?1".to_string(),
		];
		let macro_def = MacroDefinition::new("test".to_string(), body);

		let args = vec!["A".to_string(), "#$42".to_string()];
		let expanded = macro_def.expand(&args).unwrap();

		assert_eq!(expanded[0], "arg_count: 2");
		assert!(expanded[1].starts_with("local_label"));
		assert!(expanded[1].ends_with(":"));
		assert_eq!(expanded[2], "arg1_type: 1"); // ARG_REG for "A"
	}

	#[test]
	fn test_argument_type_detection() {
		let macro_def = MacroDefinition::new("test".to_string(), vec![]);

		assert_eq!(macro_def.determine_argument_type("A"), 1); // ARG_REG
		assert_eq!(macro_def.determine_argument_type("#$42"), 2); // ARG_IMMEDIATE
		assert_eq!(macro_def.determine_argument_type("$1234"), 3); // ARG_ABSOLUTE
		assert_eq!(macro_def.determine_argument_type("[ptr]"), 4); // ARG_INDIRECT
		assert_eq!(macro_def.determine_argument_type("\"hello\""), 5); // ARG_STRING
		assert_eq!(macro_def.determine_argument_type("label"), 6); // ARG_LABEL
		assert_eq!(macro_def.determine_argument_type(""), 0); // ARG_NONE
	}

	#[test]
	fn test_local_labels() {
		let body = vec!["loop\\@:".to_string(), "BNE loop\\@".to_string()];
		let macro_def = MacroDefinition::new("test".to_string(), body);

		assert!(macro_def.uses_local_labels());

		let expanded = macro_def.expand(&[]).unwrap();

		// Both lines should have the same local ID
		let id1 = expanded[0].trim_end_matches(':').split('@').last().unwrap();
		let id2 = expanded[1].split('@').last().unwrap();
		assert_eq!(id1, id2);
	}

	#[test]
	fn test_too_many_arguments() {
		let body = vec!["LDA \\1".to_string()];
		let macro_def = MacroDefinition::new("test".to_string(), body);

		let args = vec!["#$42".to_string(), "$0200".to_string()];
		let result = macro_def.expand(&args);

		assert!(result.is_err());
	}
}
