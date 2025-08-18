//! Assembly language parsing for the NES assembler.
//!
//! This module provides comprehensive parsing capabilities for 6502 assembly
//! language syntax, including instructions, directives, expressions, and
//! various assembler constructs.

pub mod directives;
pub mod expressions;
pub mod lexer;
pub mod parser;
pub mod tokens;

// Re-exports for convenience
pub use directives::{Directive, DirectiveParser, DirectiveType};
pub use expressions::{Expression, ExpressionParser, ExpressionValue};
pub use lexer::{Lexer, LexerError};
pub use parser::{Parser as AssemblyParser, SourceLocation};
pub use tokens::{Token, TokenType, TokenValue};

use crate::error::{AssemblyError, AssemblyResult, SourcePos};
use crate::instructions::{AddressingMode, CompleteInstruction, Mnemonic, Operand};
use std::fmt;

/// A complete assembly statement (instruction, directive, or label)
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
	/// Label definition
	Label {
		name: String,
		pos: SourcePos,
	},
	/// Instruction
	Instruction(CompleteInstruction),
	/// Assembler directive
	Directive(Directive),
	/// Empty line or comment
	Empty,
}

impl Statement {
	/// Get the source position of this statement
	pub fn pos(&self) -> Option<&SourcePos> {
		match self {
			Self::Label {
				pos,
				..
			} => Some(pos),
			Self::Instruction(instr) => Some(&instr.source_pos),
			Self::Directive(dir) => Some(&dir.pos),
			Self::Empty => None,
		}
	}

	/// Check if this statement is empty
	pub fn is_empty(&self) -> bool {
		matches!(self, Self::Empty)
	}

	/// Check if this statement is a label
	pub fn is_label(&self) -> bool {
		matches!(self, Self::Label { .. })
	}

	/// Check if this statement is an instruction
	pub fn is_instruction(&self) -> bool {
		matches!(self, Self::Instruction(_))
	}

	/// Check if this statement is a directive
	pub fn is_directive(&self) -> bool {
		matches!(self, Self::Directive(_))
	}

	/// Get the label name if this is a label
	pub fn label_name(&self) -> Option<&str> {
		match self {
			Self::Label {
				name,
				..
			} => Some(name),
			_ => None,
		}
	}

	/// Get the instruction if this is an instruction
	pub fn instruction(&self) -> Option<&CompleteInstruction> {
		match self {
			Self::Instruction(instr) => Some(instr),
			_ => None,
		}
	}

	/// Get the directive if this is a directive
	pub fn directive(&self) -> Option<&Directive> {
		match self {
			Self::Directive(dir) => Some(dir),
			_ => None,
		}
	}
}

impl fmt::Display for Statement {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Label {
				name,
				..
			} => write!(f, "{}:", name),
			Self::Instruction(instr) => {
				write!(f, "{} ", instr.mnemonic)?;
				match &instr.operand {
					Operand::None => Ok(()),
					Operand::Immediate8(val) => write!(f, "#${:02X}", val),
					Operand::Immediate16(val) => write!(f, "#${:04X}", val),
					Operand::ZeroPage(val) => write!(f, "${:02X}", val),
					Operand::Absolute(val) => write!(f, "${:04X}", val),
					Operand::Relative(val) => write!(f, "{:+}", val),
					Operand::Symbol(name) => write!(f, "{}", name),
					Operand::Expression(expr) => write!(f, "({})", expr),
				}
			}
			Self::Directive(dir) => write!(f, "{}", dir),
			Self::Empty => write!(f, ""),
		}
	}
}

/// Parse context for maintaining state during parsing
#[derive(Debug, Clone)]
pub struct ParseContext {
	/// Current file being parsed
	pub file: String,
	/// Current line number
	pub line: usize,
	/// Current column
	pub column: usize,
	/// Whether to allow unofficial opcodes
	pub allow_unofficial: bool,
	/// Whether to be case sensitive
	pub case_sensitive: bool,
	/// Current label context for local labels
	pub current_label: Option<String>,
}

impl ParseContext {
	/// Create a new parse context
	pub fn new(file: String) -> Self {
		Self {
			file,
			line: 1,
			column: 1,
			allow_unofficial: false,
			case_sensitive: true,
			current_label: None,
		}
	}

	/// Create a source position from current context
	pub fn source_pos(&self) -> SourcePos {
		SourcePos::new(self.file.clone().into(), self.line, self.column)
	}

	/// Advance line counter
	pub fn advance_line(&mut self) {
		self.line += 1;
		self.column = 1;
	}

	/// Advance column counter
	pub fn advance_column(&mut self, count: usize) {
		self.column += count;
	}

	/// Set current label for local label context
	pub fn set_current_label(&mut self, label: Option<String>) {
		self.current_label = label;
	}

	/// Get current line number
	pub fn line_number(&self) -> usize {
		self.line
	}
}

/// Parse options for customizing parser behavior
#[derive(Debug, Clone)]
pub struct ParseOptions {
	/// Allow unofficial/illegal opcodes
	pub allow_unofficial: bool,
	/// Case sensitivity for symbols and mnemonics
	pub case_sensitive: bool,
	/// Maximum line length
	pub max_line_length: usize,
	/// Allow C-style comments (//)
	pub allow_c_comments: bool,
	/// Allow nested comments
	pub allow_nested_comments: bool,
	/// Local label prefix character
	pub local_label_prefix: char,
	/// Anonymous label characters
	pub anonymous_label_chars: Vec<char>,
}

impl Default for ParseOptions {
	fn default() -> Self {
		Self {
			allow_unofficial: false,
			case_sensitive: true,
			max_line_length: 1024,
			allow_c_comments: true,
			allow_nested_comments: false,
			local_label_prefix: '.',
			anonymous_label_chars: vec!['+', '-'],
		}
	}
}

/// High-level parsing interface
pub struct Parser {
	options: ParseOptions,
}

impl Parser {
	/// Create a new parser with default options
	pub fn new() -> Self {
		Self {
			options: ParseOptions::default(),
		}
	}

	/// Create a new parser with custom options
	pub fn with_options(options: ParseOptions) -> Self {
		Self {
			options,
		}
	}

	/// Parse a line of assembly code
	pub fn parse_line(&self, line: &str, context: &mut ParseContext) -> AssemblyResult<Statement> {
		// Check line length
		if line.len() > self.options.max_line_length {
			return Err(AssemblyError::parse(
				context.source_pos(),
				format!("Line too long ({} characters)", line.len()),
			));
		}
		if line.trim().is_empty() {
			return Ok(Statement::Empty);
		}

		// Use a simple line-based parser for now
		let mut parser = crate::parsing::parser::Parser::new();
		parser.parse_line(context.line_number(), line)
	}

	/// Parse multiple lines of assembly code
	pub fn parse_lines(&self, lines: &[String], file: String) -> AssemblyResult<Vec<Statement>> {
		let mut context = ParseContext::new(file);
		let mut statements = Vec::new();

		for line in lines {
			context.advance_line();
			let statement = self.parse_line(line, &mut context)?;
			statements.push(statement);
		}

		Ok(statements)
	}

	/// Parse an entire file
	pub fn parse_file(&self, content: &str, file: String) -> AssemblyResult<Vec<Statement>> {
		let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
		self.parse_lines(&lines, file)
	}
}

impl Default for Parser {
	fn default() -> Self {
		Self::new()
	}
}

// Type aliases for backward compatibility
pub type ParseResult<T> = AssemblyResult<T>;
pub type StatementParser = Parser;

/// Utility functions for parsing
pub mod utils {
	use super::*;

	/// Parse a numeric literal (supports $hex, %binary, decimal)
	pub fn parse_number(s: &str, pos: &SourcePos) -> AssemblyResult<i32> {
		let trimmed = s.trim();

		if trimmed.is_empty() {
			return Err(AssemblyError::parse(pos.clone(), "Empty number literal".to_string()));
		}

		// Hexadecimal ($FF, 0xFF, FFh)
		if let Some(hex_str) = trimmed.strip_prefix('$') {
			return i32::from_str_radix(hex_str, 16).map_err(|_| {
				AssemblyError::parse(pos.clone(), format!("Invalid hex number: {}", s))
			});
		}

		if let Some(hex_str) = trimmed.strip_prefix("0x").or_else(|| trimmed.strip_prefix("0X")) {
			return i32::from_str_radix(hex_str, 16).map_err(|_| {
				AssemblyError::parse(pos.clone(), format!("Invalid hex number: {}", s))
			});
		}

		if let Some(hex_str) = trimmed.strip_suffix('h').or_else(|| trimmed.strip_suffix('H')) {
			return i32::from_str_radix(hex_str, 16).map_err(|_| {
				AssemblyError::parse(pos.clone(), format!("Invalid hex number: {}", s))
			});
		}

		// Binary (%11110000, 0b11110000)
		if let Some(bin_str) = trimmed.strip_prefix('%') {
			return i32::from_str_radix(bin_str, 2).map_err(|_| {
				AssemblyError::parse(pos.clone(), format!("Invalid binary number: {}", s))
			});
		}

		if let Some(bin_str) = trimmed.strip_prefix("0b").or_else(|| trimmed.strip_prefix("0B")) {
			return i32::from_str_radix(bin_str, 2).map_err(|_| {
				AssemblyError::parse(pos.clone(), format!("Invalid binary number: {}", s))
			});
		}

		// Octal (0777)
		if trimmed.starts_with('0')
			&& trimmed.len() > 1
			&& trimmed.chars().all(|c| c.is_ascii_digit())
		{
			return i32::from_str_radix(trimmed, 8).map_err(|_| {
				AssemblyError::parse(pos.clone(), format!("Invalid octal number: {}", s))
			});
		}

		// Decimal
		trimmed
			.parse::<i32>()
			.map_err(|_| AssemblyError::parse(pos.clone(), format!("Invalid number: {}", s)))
	}

	/// Check if a string is a valid identifier
	pub fn is_valid_identifier(s: &str) -> bool {
		if s.is_empty() {
			return false;
		}

		let first_char = s.chars().next().unwrap();
		if !first_char.is_ascii_alphabetic() && first_char != '_' && first_char != '.' {
			return false;
		}

		s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
	}

	/// Normalize identifier case
	pub fn normalize_identifier(s: &str, case_sensitive: bool) -> String {
		if case_sensitive {
			s.to_string()
		} else {
			s.to_uppercase()
		}
	}

	/// Parse string literal with escape sequences
	pub fn parse_string_literal(s: &str, pos: &SourcePos) -> AssemblyResult<String> {
		if !s.starts_with('"') || !s.ends_with('"') {
			return Err(AssemblyError::parse(
				pos.clone(),
				"String literal must be quoted".to_string(),
			));
		}

		let content = &s[1..s.len() - 1];
		let mut result = String::new();
		let mut chars = content.chars();

		while let Some(ch) = chars.next() {
			if ch == '\\' {
				match chars.next() {
					Some('n') => result.push('\n'),
					Some('r') => result.push('\r'),
					Some('t') => result.push('\t'),
					Some('\\') => result.push('\\'),
					Some('"') => result.push('"'),
					Some('0') => result.push('\0'),
					Some(c) => {
						return Err(AssemblyError::parse(
							pos.clone(),
							format!("Invalid escape sequence: \\{}", c),
						));
					}
					None => {
						return Err(AssemblyError::parse(
							pos.clone(),
							"Unterminated escape sequence".to_string(),
						));
					}
				}
			} else {
				result.push(ch);
			}
		}

		Ok(result)
	}

	/// Skip whitespace and return the rest of the string
	pub fn skip_whitespace(s: &str) -> &str {
		s.trim_start()
	}

	/// Check if a character starts a comment
	pub fn is_comment_start(ch: char, allow_c_comments: bool) -> bool {
		ch == ';' || (allow_c_comments && ch == '/')
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
	fn test_parse_number() {
		let pos = test_pos();

		// Decimal
		assert_eq!(utils::parse_number("42", &pos).unwrap(), 42);
		assert_eq!(utils::parse_number("0", &pos).unwrap(), 0);

		// Hexadecimal
		assert_eq!(utils::parse_number("$FF", &pos).unwrap(), 255);
		assert_eq!(utils::parse_number("0xFF", &pos).unwrap(), 255);
		assert_eq!(utils::parse_number("FFh", &pos).unwrap(), 255);

		// Binary
		assert_eq!(utils::parse_number("%11111111", &pos).unwrap(), 255);
		assert_eq!(utils::parse_number("0b11111111", &pos).unwrap(), 255);

		// Invalid numbers
		assert!(utils::parse_number("", &pos).is_err());
		assert!(utils::parse_number("xyz", &pos).is_err());
		assert!(utils::parse_number("$GG", &pos).is_err());
	}

	#[test]
	fn test_valid_identifier() {
		assert!(utils::is_valid_identifier("label"));
		assert!(utils::is_valid_identifier("_start"));
		assert!(utils::is_valid_identifier(".local"));
		assert!(utils::is_valid_identifier("test123"));

		assert!(!utils::is_valid_identifier(""));
		assert!(!utils::is_valid_identifier("123label"));
		assert!(!utils::is_valid_identifier("label-name"));
		assert!(!utils::is_valid_identifier("label with space"));
	}

	#[test]
	fn test_normalize_identifier() {
		assert_eq!(utils::normalize_identifier("Label", true), "Label");
		assert_eq!(utils::normalize_identifier("Label", false), "LABEL");
	}

	#[test]
	fn test_parse_string_literal() {
		let pos = test_pos();

		assert_eq!(utils::parse_string_literal("\"hello\"", &pos).unwrap(), "hello");
		assert_eq!(utils::parse_string_literal("\"\"", &pos).unwrap(), "");
		assert_eq!(utils::parse_string_literal("\"line\\nbreak\"", &pos).unwrap(), "line\nbreak");
		assert_eq!(utils::parse_string_literal("\"quote\\\"test\"", &pos).unwrap(), "quote\"test");

		assert!(utils::parse_string_literal("hello", &pos).is_err());
		assert!(utils::parse_string_literal("\"unterminated", &pos).is_err());
	}

	#[test]
	fn test_statement_types() {
		let pos = test_pos();

		let label = Statement::Label {
			name: "start".to_string(),
			pos: pos.clone(),
		};
		assert!(label.is_label());
		assert!(!label.is_instruction());
		assert_eq!(label.label_name(), Some("start"));

		let empty = Statement::Empty;
		assert!(empty.is_empty());
		assert!(empty.pos().is_none());
	}

	#[test]
	fn test_parse_context() {
		let mut ctx = ParseContext::new("test.asm".to_string());
		assert_eq!(ctx.line, 1);
		assert_eq!(ctx.column, 1);

		ctx.advance_line();
		assert_eq!(ctx.line, 2);
		assert_eq!(ctx.column, 1);

		ctx.advance_column(5);
		assert_eq!(ctx.column, 6);
	}

	#[test]
	fn test_parse_options() {
		let options = ParseOptions::default();
		assert!(!options.allow_unofficial);
		assert!(options.case_sensitive);
		assert_eq!(options.local_label_prefix, '.');
	}
}
