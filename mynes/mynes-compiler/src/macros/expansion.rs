//! Macro expansion functionality for the NES compiler
//!
//! This module handles the expansion of macros during assembly, including
//! context management and recursive expansion handling.

use crate::error::{AssemblyResult, MacroError, SourcePos};

/// Context for macro expansion
#[derive(Debug, Clone)]
pub struct MacroExpansionContext {
	/// Name of the macro being expanded
	pub macro_name: String,
	/// Arguments passed to the macro
	pub arguments: Vec<String>,
	/// Current expansion line number within the macro
	pub current_line: usize,
	/// Unique expansion ID for local label generation
	pub expansion_id: u32,
	/// Source location where macro was invoked
	pub source_location: Option<SourceLocation>,
}

/// Source location information for macro invocation
#[derive(Debug, Clone, Copy)]
pub struct SourceLocation {
	/// Source file line number
	pub line: usize,
	/// Source file name or identifier
	pub file_id: usize,
	/// Column position in the source line
	pub column: usize,
}

impl MacroExpansionContext {
	/// Create a new macro expansion context
	pub fn new(macro_name: String, arguments: Vec<String>) -> Self {
		Self {
			macro_name,
			arguments,
			current_line: 0,
			expansion_id: Self::generate_expansion_id(),
			source_location: None,
		}
	}

	/// Create a context with source location information
	pub fn with_location(
		macro_name: String,
		arguments: Vec<String>,
		location: SourceLocation,
	) -> Self {
		Self {
			macro_name,
			arguments,
			current_line: 0,
			expansion_id: Self::generate_expansion_id(),
			source_location: Some(location),
		}
	}

	/// Generate a unique expansion ID
	fn generate_expansion_id() -> u32 {
		use std::sync::atomic::{AtomicU32, Ordering};
		static COUNTER: AtomicU32 = AtomicU32::new(1);
		COUNTER.fetch_add(1, Ordering::Relaxed)
	}

	/// Get the number of arguments
	pub fn argument_count(&self) -> usize {
		self.arguments.len()
	}

	/// Get an argument by index (1-based)
	pub fn get_argument(&self, index: usize) -> Option<&str> {
		if index > 0 && index <= self.arguments.len() {
			Some(&self.arguments[index - 1])
		} else {
			None
		}
	}

	/// Advance to the next line in the macro
	pub fn advance_line(&mut self) {
		self.current_line += 1;
	}

	/// Get the current line number
	pub fn current_line(&self) -> usize {
		self.current_line
	}

	/// Get the expansion ID
	pub fn expansion_id(&self) -> u32 {
		self.expansion_id
	}

	/// Check if we have source location information
	pub fn has_source_location(&self) -> bool {
		self.source_location.is_some()
	}

	/// Get the source location
	pub fn source_location(&self) -> Option<SourceLocation> {
		self.source_location
	}
}

/// Macro expander that handles the expansion process
#[derive(Debug)]
pub struct MacroExpander {
	/// Stack of expansion contexts for nested macros
	expansion_stack: Vec<MacroExpansionContext>,
	/// Maximum allowed expansion depth
	max_depth: usize,
	/// Statistics about macro expansions
	stats: ExpansionStats,
}

/// Statistics about macro expansion
#[derive(Debug, Default, Clone)]
pub struct ExpansionStats {
	/// Total number of macros expanded
	pub macros_expanded: usize,
	/// Total number of lines generated
	pub lines_generated: usize,
	/// Maximum expansion depth reached
	pub max_depth_reached: usize,
	/// Number of recursive expansions
	pub recursive_expansions: usize,
}

impl MacroExpander {
	/// Create a new macro expander
	pub fn new() -> Self {
		Self {
			expansion_stack: Vec::new(),
			max_depth: 32,
			stats: ExpansionStats::default(),
		}
	}

	/// Set the maximum expansion depth
	pub fn set_max_depth(&mut self, depth: usize) {
		self.max_depth = depth;
	}

	/// Get the current expansion depth
	pub fn current_depth(&self) -> usize {
		self.expansion_stack.len()
	}

	/// Check if currently expanding macros
	pub fn is_expanding(&self) -> bool {
		!self.expansion_stack.is_empty()
	}

	/// Begin expansion of a macro
	pub fn begin_expansion(&mut self, context: MacroExpansionContext) -> AssemblyResult<()> {
		// Check for maximum depth
		if self.expansion_stack.len() >= self.max_depth {
			return Err(MacroError::ExpansionError {
				pos: SourcePos::file_only(std::path::PathBuf::from("<macro>")),
				message: format!("Maximum macro expansion depth ({}) exceeded", self.max_depth),
			}
			.into());
		}

		// Check for recursive macro calls
		if self.expansion_stack.iter().any(|ctx| ctx.macro_name == context.macro_name) {
			self.stats.recursive_expansions += 1;
			return Err(MacroError::ExpansionError {
				pos: SourcePos::file_only(std::path::PathBuf::from("<macro>")),
				message: format!("Recursive macro expansion detected: {}", context.macro_name),
			}
			.into());
		}

		// Update statistics
		self.stats.macros_expanded += 1;
		self.stats.max_depth_reached = self.stats.max_depth_reached.max(self.current_depth() + 1);

		// Push the new context
		self.expansion_stack.push(context);

		Ok(())
	}

	/// End the current macro expansion
	pub fn end_expansion(&mut self) -> Option<MacroExpansionContext> {
		self.expansion_stack.pop()
	}

	/// Get the current expansion context
	pub fn current_context(&self) -> Option<&MacroExpansionContext> {
		self.expansion_stack.last()
	}

	/// Get the current expansion context (mutable)
	pub fn current_context_mut(&mut self) -> Option<&mut MacroExpansionContext> {
		self.expansion_stack.last_mut()
	}

	/// Expand a single line with parameter substitution
	pub fn expand_line(&mut self, line: &str) -> AssemblyResult<String> {
		let context = self.current_context().ok_or_else(|| MacroError::ExpansionError {
			pos: SourcePos::file_only(std::path::PathBuf::from("<macro>")),
			message: "No active macro expansion context".to_string(),
		})?;

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

							if let Some(arg) = context.get_argument(param_index) {
								result.push_str(arg);
							}
							// If parameter not provided, substitute with empty string
						}
						// Parameter count \#
						'#' => {
							chars.next(); // consume the #
							result.push_str(&context.argument_count().to_string());
						}
						// Local label unique ID \@
						'@' => {
							chars.next(); // consume the @
							result.push_str(&context.expansion_id().to_string());
						}
						// Parameter type queries \?1-\?9
						'?' => {
							chars.next(); // consume the ?
							if let Some(&digit_ch) = chars.peek() {
								if digit_ch.is_ascii_digit() {
									chars.next(); // consume the digit
									let param_index = digit_ch.to_digit(10).unwrap() as usize;

									if let Some(arg) = context.get_argument(param_index) {
										let arg_type = Self::determine_argument_type(arg);
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

		// Update statistics
		self.stats.lines_generated += 1;

		// Advance line number in current context
		if let Some(context) = self.current_context_mut() {
			context.advance_line();
		}

		Ok(result)
	}

	/// Determine the type of a macro argument
	fn determine_argument_type(arg: &str) -> u8 {
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

	/// Get expansion statistics
	pub fn stats(&self) -> &ExpansionStats {
		&self.stats
	}

	/// Reset expansion statistics
	pub fn reset_stats(&mut self) {
		self.stats = ExpansionStats::default();
	}

	/// Clear all expansion state
	pub fn clear(&mut self) {
		self.expansion_stack.clear();
		self.reset_stats();
	}

	/// Get all expansion contexts (for debugging)
	pub fn expansion_stack(&self) -> &[MacroExpansionContext] {
		&self.expansion_stack
	}

	/// Check if a macro is currently being expanded
	pub fn is_macro_expanding(&self, macro_name: &str) -> bool {
		self.expansion_stack.iter().any(|ctx| ctx.macro_name == macro_name)
	}
}

impl Default for MacroExpander {
	fn default() -> Self {
		Self::new()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_expansion_context_creation() {
		let context = MacroExpansionContext::new(
			"test_macro".to_string(),
			vec!["arg1".to_string(), "arg2".to_string()],
		);

		assert_eq!(context.macro_name, "test_macro");
		assert_eq!(context.argument_count(), 2);
		assert_eq!(context.get_argument(1), Some("arg1"));
		assert_eq!(context.get_argument(2), Some("arg2"));
		assert_eq!(context.get_argument(3), None);
	}

	#[test]
	fn test_expander_creation() {
		let expander = MacroExpander::new();
		assert_eq!(expander.current_depth(), 0);
		assert!(!expander.is_expanding());
	}

	#[test]
	fn test_begin_end_expansion() {
		let mut expander = MacroExpander::new();
		let context = MacroExpansionContext::new("test".to_string(), vec![]);

		assert!(expander.begin_expansion(context).is_ok());
		assert_eq!(expander.current_depth(), 1);
		assert!(expander.is_expanding());

		let ended_context = expander.end_expansion();
		assert!(ended_context.is_some());
		assert_eq!(expander.current_depth(), 0);
		assert!(!expander.is_expanding());
	}

	#[test]
	fn test_line_expansion() {
		let mut expander = MacroExpander::new();
		let context = MacroExpansionContext::new(
			"test".to_string(),
			vec!["#$42".to_string(), "$0200".to_string()],
		);

		expander.begin_expansion(context).unwrap();

		let expanded = expander.expand_line("LDA \\1").unwrap();
		assert_eq!(expanded, "LDA #$42");

		let expanded = expander.expand_line("STA \\2").unwrap();
		assert_eq!(expanded, "STA $0200");
	}

	#[test]
	fn test_special_parameters() {
		let mut expander = MacroExpander::new();
		let context = MacroExpansionContext::new(
			"test".to_string(),
			vec!["A".to_string(), "#$42".to_string()],
		);

		expander.begin_expansion(context).unwrap();

		let expanded = expander.expand_line("arg_count: \\#").unwrap();
		assert_eq!(expanded, "arg_count: 2");

		let expanded = expander.expand_line("local_label\\@:").unwrap();
		assert!(expanded.starts_with("local_label"));
		assert!(expanded.ends_with(":"));

		let expanded = expander.expand_line("arg1_type: \\?1").unwrap();
		assert_eq!(expanded, "arg1_type: 1"); // ARG_REG for "A"
	}

	#[test]
	fn test_recursive_expansion_detection() {
		let mut expander = MacroExpander::new();
		let context1 = MacroExpansionContext::new("recursive".to_string(), vec![]);
		let context2 = MacroExpansionContext::new("recursive".to_string(), vec![]);

		assert!(expander.begin_expansion(context1).is_ok());
		assert!(expander.begin_expansion(context2).is_err());
	}

	#[test]
	fn test_max_depth_limit() {
		let mut expander = MacroExpander::new();
		expander.set_max_depth(2);

		let context1 = MacroExpansionContext::new("macro1".to_string(), vec![]);
		let context2 = MacroExpansionContext::new("macro2".to_string(), vec![]);
		let context3 = MacroExpansionContext::new("macro3".to_string(), vec![]);

		assert!(expander.begin_expansion(context1).is_ok());
		assert!(expander.begin_expansion(context2).is_ok());
		assert!(expander.begin_expansion(context3).is_err());
	}

	#[test]
	fn test_argument_type_detection() {
		assert_eq!(MacroExpander::determine_argument_type("A"), 1); // ARG_REG
		assert_eq!(MacroExpander::determine_argument_type("#$42"), 2); // ARG_IMMEDIATE
		assert_eq!(MacroExpander::determine_argument_type("$1234"), 3); // ARG_ABSOLUTE
		assert_eq!(MacroExpander::determine_argument_type("[ptr]"), 4); // ARG_INDIRECT
		assert_eq!(MacroExpander::determine_argument_type("\"hello\""), 5); // ARG_STRING
		assert_eq!(MacroExpander::determine_argument_type("label"), 6); // ARG_LABEL
		assert_eq!(MacroExpander::determine_argument_type(""), 0); // ARG_NONE
	}
}
