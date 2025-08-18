//! Parser implementation for the NES compiler
//!
//! This module provides the main parser that converts tokenized input
//! into abstract syntax tree (AST) nodes for assembly processing.

use crate::error::{AssemblyError, AssemblyResult};
use crate::parsing::Statement;

/// Source location information
#[derive(Debug, Clone)]
pub struct SourceLocation {
	pub line: usize,
	pub column: usize,
	pub offset: usize,
}

impl SourceLocation {
	pub fn new(line: usize, column: usize, offset: usize) -> Self {
		Self {
			line,
			column,
			offset,
		}
	}
}

/// Main parser for assembly source code
#[derive(Debug)]
pub struct Parser {
	/// Current source file being parsed
	current_file: String,
	/// Current line number
	current_line: usize,
	/// Parser configuration
	config: ParserConfig,
}

/// Configuration options for the parser
#[derive(Debug, Clone)]
pub struct ParserConfig {
	/// Whether to allow case-insensitive instruction names
	pub case_insensitive: bool,
	/// Whether to allow extended syntax features
	pub allow_extensions: bool,
	/// Maximum nesting depth for expressions
	pub max_expression_depth: usize,
}

impl Default for ParserConfig {
	fn default() -> Self {
		Self {
			case_insensitive: true,
			allow_extensions: false,
			max_expression_depth: 32,
		}
	}
}

impl Parser {
	/// Create a new parser with default configuration
	pub fn new() -> Self {
		Self {
			current_file: String::new(),
			current_line: 0,
			config: ParserConfig::default(),
		}
	}

	/// Create a new parser with custom configuration
	pub fn with_config(config: ParserConfig) -> Self {
		Self {
			current_file: String::new(),
			current_line: 0,
			config,
		}
	}

	/// Set the current file being parsed
	pub fn set_current_file(&mut self, filename: String) {
		self.current_file = filename;
		self.current_line = 0;
	}

	/// Get the current source location
	pub fn current_location(&self) -> SourceLocation {
		SourceLocation::new(self.current_line, 1, 0)
	}

	/// Read a file and return all lines
	pub fn read_file(&mut self, path: &std::path::Path) -> AssemblyResult<Vec<String>> {
		use std::fs;
		use std::io::{self, BufRead, BufReader};

		self.set_current_file(path.to_string_lossy().to_string());

		let file = fs::File::open(path).map_err(|e| AssemblyError::Io {
			pos: None,
			source: e,
		})?;

		let reader = BufReader::new(file);
		let mut lines = Vec::new();

		for line_result in reader.lines() {
			match line_result {
				Ok(line) => lines.push(line),
				Err(e) => {
					return Err(AssemblyError::Io {
						pos: None,
						source: e,
					});
				}
			}
		}

		Ok(lines)
	}

	/// Parse a single line of assembly code
	pub fn parse_line(&mut self, line_number: usize, line: &str) -> AssemblyResult<Statement> {
		self.current_line = line_number;

		// Skip empty lines and comments
		let trimmed = line.trim();
		if trimmed.is_empty() || trimmed.starts_with(';') {
			return Ok(Statement::Empty);
		}

		// This is a simplified parser implementation
		// A full implementation would use proper tokenization and parsing

		// Check for labels (ending with ':')
		if let Some(colon_pos) = trimmed.find(':') {
			let label_part = trimmed[..colon_pos].trim();
			if !label_part.is_empty() && self.is_valid_label(label_part) {
				return Ok(Statement::Empty); // Simplified for now
			}
		}

		// Check for directives (starting with '.')
		if trimmed.starts_with('.') {
			return self.parse_directive(trimmed);
		}

		// Check for assignments (containing '=')
		if let Some(eq_pos) = trimmed.find('=') {
			let symbol = trimmed[..eq_pos].trim().to_string();
			let value_str = trimmed[eq_pos + 1..].trim();

			if !symbol.is_empty() && self.is_valid_symbol(&symbol) {
				// For now, return empty statement
				return Ok(Statement::Empty);
			}
		}

		// For now, treat everything else as empty
		Ok(Statement::Empty)
	}

	/// Parse a directive line
	fn parse_directive(&self, line: &str) -> AssemblyResult<Statement> {
		// This is a simplified directive parser
		// A full implementation would handle all NES-specific directives

		let parts: Vec<&str> = line.split_whitespace().collect();
		if parts.is_empty() {
			return Err(AssemblyError::parse(
				crate::error::SourcePos::new(
					std::path::PathBuf::from(&self.current_file),
					self.current_line,
					1,
				),
				"Empty directive".to_string(),
			));
		}

		let directive_name = parts[0].to_uppercase();

		match directive_name.as_str() {
			".ORG" => {
				if parts.len() != 2 {
					return Err(AssemblyError::parse(
						crate::error::SourcePos::new(
							std::path::PathBuf::from(&self.current_file),
							self.current_line,
							1,
						),
						".ORG requires exactly one argument".to_string(),
					));
				}

				// Parse address (simplified)
				let _address = self.parse_number(parts[1])?;
				Ok(Statement::Empty)
			}
			".DB" => Ok(Statement::Empty),
			".DW" => Ok(Statement::Empty),
			_ => {
				// For now, return a generic directive
				Err(AssemblyError::parse(
					crate::error::SourcePos::new(
						std::path::PathBuf::from(&self.current_file),
						self.current_line,
						1,
					),
					format!("Unsupported directive: {}", directive_name),
				))
			}
		}
	}

	/// Parse a numeric value (decimal, hex, binary)
	fn parse_number(&self, input: &str) -> AssemblyResult<i32> {
		let trimmed = input.trim();

		if trimmed.starts_with('$') {
			// Hexadecimal
			i32::from_str_radix(&trimmed[1..], 16).map_err(|_| {
				AssemblyError::parse(
					crate::error::SourcePos::new(
						std::path::PathBuf::from(&self.current_file),
						self.current_line,
						1,
					),
					format!("Invalid hex literal: {}", input),
				)
			})
		} else if trimmed.starts_with('%') {
			// Binary
			i32::from_str_radix(&trimmed[1..], 2).map_err(|_| {
				AssemblyError::parse(
					crate::error::SourcePos::new(
						std::path::PathBuf::from(&self.current_file),
						self.current_line,
						1,
					),
					format!("Invalid binary literal: {}", input),
				)
			})
		} else {
			// Decimal
			trimmed.parse::<i32>().map_err(|_| {
				AssemblyError::parse(
					crate::error::SourcePos::new(
						std::path::PathBuf::from(&self.current_file),
						self.current_line,
						1,
					),
					format!("Invalid decimal literal: {}", input),
				)
			})
		}
	}

	/// Check if a string is a valid label name
	fn is_valid_label(&self, name: &str) -> bool {
		if name.is_empty() {
			return false;
		}

		let first_char = name.chars().next().unwrap();
		if !first_char.is_ascii_alphabetic() && first_char != '_' && first_char != '.' {
			return false;
		}

		name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
	}

	/// Check if a string is a valid symbol name
	fn is_valid_symbol(&self, name: &str) -> bool {
		self.is_valid_label(name)
	}

	/// Reset parser state
	pub fn reset(&mut self) {
		self.current_line = 0;
	}

	/// Get the current configuration
	pub fn config(&self) -> &ParserConfig {
		&self.config
	}

	/// Set the parser configuration
	pub fn set_config(&mut self, config: ParserConfig) {
		self.config = config;
	}
}

impl Default for Parser {
	fn default() -> Self {
		Self::new()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::io::Write;
	use tempfile::NamedTempFile;

	#[test]
	fn test_parser_creation() {
		let parser = Parser::new();
		assert_eq!(parser.current_line, 0);
		assert!(parser.config.case_insensitive);
	}

	#[test]
	fn test_label_validation() {
		let parser = Parser::new();

		assert!(parser.is_valid_label("label"));
		assert!(parser.is_valid_label("_start"));
		assert!(parser.is_valid_label(".local"));
		assert!(parser.is_valid_label("test123"));

		assert!(!parser.is_valid_label(""));
		assert!(!parser.is_valid_label("123label"));
		assert!(!parser.is_valid_label("label-name"));
	}

	#[test]
	fn test_number_parsing() {
		let parser = Parser::new();

		assert_eq!(parser.parse_number("42").unwrap(), 42);
		assert_eq!(parser.parse_number("$FF").unwrap(), 255);
		assert_eq!(parser.parse_number("%11111111").unwrap(), 255);

		assert!(parser.parse_number("invalid").is_err());
		assert!(parser.parse_number("$GG").is_err());
	}

	#[test]
	fn test_comment_parsing() {
		let mut parser = Parser::new();

		let result = parser.parse_line(1, "; This is a comment").unwrap();
		match result {
			Statement::Empty => {} // Success
			_ => panic!("Expected empty statement"),
		}
	}

	#[test]
	fn test_label_parsing() {
		let mut parser = Parser::new();

		let result = parser.parse_line(1, "start:").unwrap();
		match result {
			Statement::Empty => {} // Success
			_ => panic!("Expected empty statement"),
		}
	}

	#[test]
	fn test_file_reading() {
		let mut temp_file = NamedTempFile::new().unwrap();
		writeln!(temp_file, "start:").unwrap();
		writeln!(temp_file, "    LDA #$42").unwrap();
		writeln!(temp_file, "; Comment").unwrap();

		let mut parser = Parser::new();
		let lines = parser.read_file(temp_file.path()).unwrap();

		assert_eq!(lines.len(), 3);
		assert_eq!(lines[0], "start:");
		assert_eq!(lines[1], "    LDA #$42");
		assert_eq!(lines[2], "; Comment");
	}
}
