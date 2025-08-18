//! Lexical analyzer for assembly language tokenization.
//!
//! This module provides tokenization of assembly source code, breaking down
//! text into meaningful tokens for the parser to process.

use crate::error::{AssemblyError, AssemblyResult, SourcePos};
use crate::parsing::{ParseContext, ParseOptions};
use std::fmt;

/// Token types supported by the lexer
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
	/// Identifier (labels, symbols, mnemonics)
	Identifier,
	/// Numeric literal
	Number,
	/// String literal
	String,
	/// Character literal
	Character,
	/// Operator or punctuation
	Operator,
	/// Comment
	Comment,
	/// Whitespace
	Whitespace,
	/// End of line
	EndOfLine,
	/// End of file
	EndOfFile,
}

impl fmt::Display for TokenType {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let name = match self {
			Self::Identifier => "identifier",
			Self::Number => "number",
			Self::String => "string",
			Self::Character => "character",
			Self::Operator => "operator",
			Self::Comment => "comment",
			Self::Whitespace => "whitespace",
			Self::EndOfLine => "end of line",
			Self::EndOfFile => "end of file",
		};
		write!(f, "{}", name)
	}
}

/// Token value containing the parsed content
#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue {
	/// String value for identifiers, strings, comments
	String(String),
	/// Numeric value
	Number(i32),
	/// Character value
	Character(char),
	/// Operator character
	Operator(char),
	/// No value (for whitespace, EOF, etc.)
	None,
}

impl TokenValue {
	/// Get string value if available
	pub fn as_string(&self) -> Option<&str> {
		match self {
			Self::String(s) => Some(s),
			_ => None,
		}
	}

	/// Get numeric value if available
	pub fn as_number(&self) -> Option<i32> {
		match self {
			Self::Number(n) => Some(*n),
			_ => None,
		}
	}

	/// Get character value if available
	pub fn as_char(&self) -> Option<char> {
		match self {
			Self::Character(c) => Some(*c),
			Self::Operator(c) => Some(*c),
			_ => None,
		}
	}

	/// Check if token has a value
	pub fn has_value(&self) -> bool {
		!matches!(self, Self::None)
	}
}

impl fmt::Display for TokenValue {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::String(s) => write!(f, "{}", s),
			Self::Number(n) => write!(f, "{}", n),
			Self::Character(c) => write!(f, "'{}'", c),
			Self::Operator(c) => write!(f, "{}", c),
			Self::None => write!(f, "<none>"),
		}
	}
}

/// A token produced by the lexer
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
	/// Token type
	pub token_type: TokenType,
	/// Token value
	pub value: TokenValue,
	/// Source position
	pub pos: SourcePos,
	/// Original text that produced this token
	pub text: String,
}

impl Token {
	/// Create a new token
	pub fn new(token_type: TokenType, value: TokenValue, pos: SourcePos, text: String) -> Self {
		Self {
			token_type,
			value,
			pos,
			text,
		}
	}

	/// Create an identifier token
	pub fn identifier(name: String, pos: SourcePos) -> Self {
		Self::new(TokenType::Identifier, TokenValue::String(name.clone()), pos, name)
	}

	/// Create a number token
	pub fn number(value: i32, pos: SourcePos, text: String) -> Self {
		Self::new(TokenType::Number, TokenValue::Number(value), pos, text)
	}

	/// Create a string token
	pub fn string(value: String, pos: SourcePos, text: String) -> Self {
		Self::new(TokenType::String, TokenValue::String(value), pos, text)
	}

	/// Create an operator token
	pub fn operator(op: char, pos: SourcePos) -> Self {
		Self::new(TokenType::Operator, TokenValue::Operator(op), pos, op.to_string())
	}

	/// Create a comment token
	pub fn comment(text: String, pos: SourcePos) -> Self {
		Self::new(TokenType::Comment, TokenValue::String(text.clone()), pos, text)
	}

	/// Create a whitespace token
	pub fn whitespace(text: String, pos: SourcePos) -> Self {
		Self::new(TokenType::Whitespace, TokenValue::None, pos, text)
	}

	/// Create an end-of-line token
	pub fn end_of_line(pos: SourcePos) -> Self {
		Self::new(TokenType::EndOfLine, TokenValue::None, pos, "\n".to_string())
	}

	/// Create an end-of-file token
	pub fn end_of_file(pos: SourcePos) -> Self {
		Self::new(TokenType::EndOfFile, TokenValue::None, pos, String::new())
	}

	/// Check if this token is significant (not whitespace or comment)
	pub fn is_significant(&self) -> bool {
		!matches!(
			self.token_type,
			TokenType::Whitespace | TokenType::Comment | TokenType::EndOfLine
		)
	}

	/// Get the length of the token text
	pub fn len(&self) -> usize {
		self.text.len()
	}

	/// Check if token is empty
	pub fn is_empty(&self) -> bool {
		self.text.is_empty()
	}
}

impl fmt::Display for Token {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{} ({})", self.token_type, self.value)
	}
}

/// Lexer error types
#[derive(Debug, Clone)]
pub enum LexerError {
	/// Invalid character in input
	InvalidCharacter {
		ch: char,
		pos: SourcePos,
	},
	/// Unterminated string literal
	UnterminatedString {
		pos: SourcePos,
	},
	/// Unterminated character literal
	UnterminatedCharacter {
		pos: SourcePos,
	},
	/// Invalid number format
	InvalidNumber {
		text: String,
		pos: SourcePos,
	},
	/// Invalid escape sequence
	InvalidEscape {
		sequence: String,
		pos: SourcePos,
	},
	/// Line too long
	LineTooLong {
		length: usize,
		pos: SourcePos,
	},
}

impl fmt::Display for LexerError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::InvalidCharacter {
				ch,
				pos,
			} => {
				write!(f, "Invalid character '{}' at {}", ch, pos)
			}
			Self::UnterminatedString {
				pos,
			} => {
				write!(f, "Unterminated string literal at {}", pos)
			}
			Self::UnterminatedCharacter {
				pos,
			} => {
				write!(f, "Unterminated character literal at {}", pos)
			}
			Self::InvalidNumber {
				text,
				pos,
			} => {
				write!(f, "Invalid number '{}' at {}", text, pos)
			}
			Self::InvalidEscape {
				sequence,
				pos,
			} => {
				write!(f, "Invalid escape sequence '{}' at {}", sequence, pos)
			}
			Self::LineTooLong {
				length,
				pos,
			} => {
				write!(f, "Line too long ({} characters) at {}", length, pos)
			}
		}
	}
}

impl std::error::Error for LexerError {}

/// Lexical analyzer for assembly source code
pub struct Lexer<'a> {
	/// Input text
	input: &'a str,
	/// Current position in input
	position: usize,
	/// Current line number
	line: usize,
	/// Current column number
	column: usize,
	/// Parse options
	options: &'a ParseOptions,
}

impl<'a> Lexer<'a> {
	/// Create a new lexer
	pub fn new(input: &'a str, options: &'a ParseOptions) -> Self {
		Self {
			input,
			position: 0,
			line: 1,
			column: 1,
			options,
		}
	}

	/// Tokenize the entire input
	pub fn tokenize(&mut self, context: &ParseContext) -> AssemblyResult<Vec<Token>> {
		let mut tokens = Vec::new();

		while !self.is_at_end() {
			let token = self.next_token(context)?;
			tokens.push(token);
		}

		// Add end-of-file token
		tokens.push(Token::end_of_file(self.current_pos(context)));

		Ok(tokens)
	}

	/// Get the next token from the input
	pub fn next_token(&mut self, context: &ParseContext) -> AssemblyResult<Token> {
		self.skip_whitespace();

		if self.is_at_end() {
			return Ok(Token::end_of_file(self.current_pos(context)));
		}

		let start_pos = self.current_pos(context);
		let ch = self.current_char();

		match ch {
			// Line ending
			'\n' => {
				self.advance();
				Ok(Token::end_of_line(start_pos))
			}

			// Comments
			';' => self.scan_comment(start_pos),
			'/' if self.options.allow_c_comments && self.peek() == Some('/') => {
				self.scan_c_comment(start_pos)
			}

			// String literals
			'"' => self.scan_string(start_pos),

			// Character literals
			'\'' => self.scan_character(start_pos),

			// Numbers
			ch if ch.is_ascii_digit() => self.scan_number(start_pos),
			'$' if self.peek().map_or(false, |c| c.is_ascii_hexdigit()) => {
				self.scan_hex_number(start_pos)
			}
			'%' if self.peek().map_or(false, |c| c == '0' || c == '1') => {
				self.scan_binary_number(start_pos)
			}

			// Identifiers and keywords
			ch if ch.is_ascii_alphabetic() || ch == '_' || ch == '.' => {
				self.scan_identifier(start_pos)
			}

			// Operators and punctuation
			ch if self.is_operator_char(ch) => {
				self.advance();
				Ok(Token::operator(ch, start_pos))
			}

			// Invalid character
			ch => Err(AssemblyError::parse(start_pos, format!("Unexpected character: '{}'", ch))),
		}
	}

	/// Skip whitespace characters (except newlines)
	fn skip_whitespace(&mut self) {
		while let Some(ch) = self.current_char_opt() {
			if ch.is_whitespace() && ch != '\n' {
				self.advance();
			} else {
				break;
			}
		}
	}

	/// Scan a comment starting with semicolon
	fn scan_comment(&mut self, start_pos: SourcePos) -> AssemblyResult<Token> {
		let start = self.position;
		self.advance(); // Skip ';'

		// Read until end of line
		while let Some(ch) = self.current_char_opt() {
			if ch == '\n' {
				break;
			}
			self.advance();
		}

		let text = self.input[start..self.position].to_string();
		Ok(Token::comment(text, start_pos))
	}

	/// Scan a C-style comment starting with //
	fn scan_c_comment(&mut self, start_pos: SourcePos) -> AssemblyResult<Token> {
		let start = self.position;
		self.advance(); // Skip first '/'
		self.advance(); // Skip second '/'

		// Read until end of line
		while let Some(ch) = self.current_char_opt() {
			if ch == '\n' {
				break;
			}
			self.advance();
		}

		let text = self.input[start..self.position].to_string();
		Ok(Token::comment(text, start_pos))
	}

	/// Scan a string literal
	fn scan_string(&mut self, start_pos: SourcePos) -> AssemblyResult<Token> {
		let start = self.position;
		self.advance(); // Skip opening quote

		let mut value = String::new();
		let mut escaped = false;

		while let Some(ch) = self.current_char_opt() {
			if escaped {
				value.push(self.parse_escape_sequence(ch, &start_pos)?);
				escaped = false;
			} else if ch == '\\' {
				escaped = true;
			} else if ch == '"' {
				self.advance(); // Skip closing quote
				let text = self.input[start..self.position].to_string();
				return Ok(Token::string(value, start_pos, text));
			} else if ch == '\n' {
				return Err(AssemblyError::parse(
					start_pos,
					"Unterminated string literal".to_string(),
				));
			} else {
				value.push(ch);
			}
			self.advance();
		}

		Err(AssemblyError::parse(start_pos, "Unterminated string literal".to_string()))
	}

	/// Scan a character literal
	fn scan_character(&mut self, start_pos: SourcePos) -> AssemblyResult<Token> {
		let start = self.position;
		self.advance(); // Skip opening quote

		let ch = self.current_char_opt().ok_or_else(|| {
			AssemblyError::parse(start_pos.clone(), "Unterminated character literal".to_string())
		})?;

		let value = if ch == '\\' {
			self.advance();
			let escaped_ch = self.current_char_opt().ok_or_else(|| {
				AssemblyError::parse(
					start_pos.clone(),
					"Unterminated character literal".to_string(),
				)
			})?;
			self.parse_escape_sequence(escaped_ch, &start_pos)?
		} else {
			ch
		};

		self.advance(); // Skip character

		if self.current_char_opt() != Some('\'') {
			return Err(AssemblyError::parse(
				start_pos,
				"Unterminated character literal".to_string(),
			));
		}

		self.advance(); // Skip closing quote
		let text = self.input[start..self.position].to_string();
		Ok(Token::new(TokenType::Character, TokenValue::Character(value), start_pos, text))
	}

	/// Parse escape sequence in string/character literal
	fn parse_escape_sequence(&self, ch: char, pos: &SourcePos) -> AssemblyResult<char> {
		match ch {
			'n' => Ok('\n'),
			'r' => Ok('\r'),
			't' => Ok('\t'),
			'\\' => Ok('\\'),
			'\'' => Ok('\''),
			'"' => Ok('"'),
			'0' => Ok('\0'),
			_ => {
				Err(AssemblyError::parse(pos.clone(), format!("Invalid escape sequence: \\{}", ch)))
			}
		}
	}

	/// Scan a decimal number
	fn scan_number(&mut self, start_pos: SourcePos) -> AssemblyResult<Token> {
		let start = self.position;

		while let Some(ch) = self.current_char_opt() {
			if ch.is_ascii_digit() {
				self.advance();
			} else {
				break;
			}
		}

		let text = &self.input[start..self.position];
		let value = text.parse::<i32>().map_err(|_| {
			AssemblyError::parse(start_pos.clone(), format!("Invalid number: {}", text))
		})?;

		Ok(Token::number(value, start_pos, text.to_string()))
	}

	/// Scan a hexadecimal number starting with $
	fn scan_hex_number(&mut self, start_pos: SourcePos) -> AssemblyResult<Token> {
		let start = self.position;
		self.advance(); // Skip '$'

		let hex_start = self.position;
		while let Some(ch) = self.current_char_opt() {
			if ch.is_ascii_hexdigit() {
				self.advance();
			} else {
				break;
			}
		}

		if self.position == hex_start {
			return Err(AssemblyError::parse(
				start_pos,
				"Invalid hex number: missing digits".to_string(),
			));
		}

		let hex_text = &self.input[hex_start..self.position];
		let value = i32::from_str_radix(hex_text, 16).map_err(|_| {
			AssemblyError::parse(start_pos.clone(), format!("Invalid hex number: {}", hex_text))
		})?;

		let text = &self.input[start..self.position];
		Ok(Token::number(value, start_pos, text.to_string()))
	}

	/// Scan a binary number starting with %
	fn scan_binary_number(&mut self, start_pos: SourcePos) -> AssemblyResult<Token> {
		let start = self.position;
		self.advance(); // Skip '%'

		let bin_start = self.position;
		while let Some(ch) = self.current_char_opt() {
			if ch == '0' || ch == '1' {
				self.advance();
			} else {
				break;
			}
		}

		if self.position == bin_start {
			return Err(AssemblyError::parse(
				start_pos,
				"Invalid binary number: missing digits".to_string(),
			));
		}

		let bin_text = &self.input[bin_start..self.position];
		let value = i32::from_str_radix(bin_text, 2).map_err(|_| {
			AssemblyError::parse(start_pos.clone(), format!("Invalid binary number: {}", bin_text))
		})?;

		let text = &self.input[start..self.position];
		Ok(Token::number(value, start_pos, text.to_string()))
	}

	/// Scan an identifier or keyword
	fn scan_identifier(&mut self, start_pos: SourcePos) -> AssemblyResult<Token> {
		let start = self.position;

		while let Some(ch) = self.current_char_opt() {
			if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' {
				self.advance();
			} else {
				break;
			}
		}

		let text = &self.input[start..self.position];
		Ok(Token::identifier(text.to_string(), start_pos))
	}

	/// Check if character is an operator
	fn is_operator_char(&self, ch: char) -> bool {
		matches!(
			ch,
			'+' | '-'
				| '*' | '/' | '%'
				| '=' | '<' | '>'
				| '!' | '&' | '|'
				| '^' | '~' | '('
				| ')' | '[' | ']'
				| '{' | '}' | ','
				| ':' | '#'
		)
	}

	/// Get current character
	fn current_char(&self) -> char {
		self.input.chars().nth(self.position).unwrap_or('\0')
	}

	/// Get current character as Option
	fn current_char_opt(&self) -> Option<char> {
		self.input.chars().nth(self.position)
	}

	/// Peek at next character
	fn peek(&self) -> Option<char> {
		self.input.chars().nth(self.position + 1)
	}

	/// Advance to next character
	fn advance(&mut self) {
		if let Some(ch) = self.current_char_opt() {
			self.position += ch.len_utf8();
			if ch == '\n' {
				self.line += 1;
				self.column = 1;
			} else {
				self.column += 1;
			}
		}
	}

	/// Check if at end of input
	fn is_at_end(&self) -> bool {
		self.position >= self.input.len()
	}

	/// Get current source position
	fn current_pos(&self, context: &ParseContext) -> SourcePos {
		SourcePos::new(context.file.clone().into(), self.line, self.column)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::path::PathBuf;

	fn test_context() -> ParseContext {
		ParseContext::new("test.asm".to_string())
	}

	fn test_options() -> ParseOptions {
		ParseOptions::default()
	}

	#[test]
	fn test_tokenize_identifier() {
		let options = test_options();
		let mut lexer = Lexer::new("start", &options);
		let context = test_context();
		let tokens = lexer.tokenize(&context).unwrap();

		assert_eq!(tokens.len(), 2); // identifier + EOF
		assert_eq!(tokens[0].token_type, TokenType::Identifier);
		assert_eq!(tokens[0].value.as_string(), Some("start"));
	}

	#[test]
	fn test_tokenize_number() {
		let options = test_options();
		let mut lexer = Lexer::new("42", &options);
		let context = test_context();
		let tokens = lexer.tokenize(&context).unwrap();

		assert_eq!(tokens.len(), 2); // number + EOF
		assert_eq!(tokens[0].token_type, TokenType::Number);
		assert_eq!(tokens[0].value.as_number(), Some(42));
	}

	#[test]
	fn test_tokenize_hex_number() {
		let options = test_options();
		let mut lexer = Lexer::new("$FF", &options);
		let context = test_context();
		let tokens = lexer.tokenize(&context).unwrap();

		assert_eq!(tokens.len(), 2); // number + EOF
		assert_eq!(tokens[0].token_type, TokenType::Number);
		assert_eq!(tokens[0].value.as_number(), Some(255));
	}

	#[test]
	fn test_tokenize_binary_number() {
		let options = test_options();
		let mut lexer = Lexer::new("%11111111", &options);
		let context = test_context();
		let tokens = lexer.tokenize(&context).unwrap();

		assert_eq!(tokens.len(), 2); // number + EOF
		assert_eq!(tokens[0].token_type, TokenType::Number);
		assert_eq!(tokens[0].value.as_number(), Some(255));
	}

	#[test]
	fn test_tokenize_string() {
		let options = test_options();
		let mut lexer = Lexer::new("\"hello world\"", &options);
		let context = test_context();
		let tokens = lexer.tokenize(&context).unwrap();

		assert_eq!(tokens.len(), 2); // string + EOF
		assert_eq!(tokens[0].token_type, TokenType::String);
		assert_eq!(tokens[0].value.as_string(), Some("hello world"));
	}

	#[test]
	fn test_tokenize_character() {
		let options = test_options();
		let mut lexer = Lexer::new("'A'", &options);
		let context = test_context();
		let tokens = lexer.tokenize(&context).unwrap();

		assert_eq!(tokens.len(), 2); // character + EOF
		assert_eq!(tokens[0].token_type, TokenType::Character);
		assert_eq!(tokens[0].value.as_char(), Some('A'));
	}

	#[test]
	fn test_tokenize_comment() {
		let options = test_options();
		let mut lexer = Lexer::new("; this is a comment", &options);
		let context = test_context();
		let tokens = lexer.tokenize(&context).unwrap();

		assert_eq!(tokens.len(), 2); // comment + EOF
		assert_eq!(tokens[0].token_type, TokenType::Comment);
	}

	#[test]
	fn test_tokenize_operators() {
		let options = test_options();
		let mut lexer = Lexer::new("+ - * / = < >", &options);
		let context = test_context();
		let tokens = lexer.tokenize(&context).unwrap();

		let operators: Vec<_> =
			tokens.iter().filter(|t| t.token_type == TokenType::Operator).collect();

		assert_eq!(operators.len(), 7);
		assert_eq!(operators[0].value.as_char(), Some('+'));
		assert_eq!(operators[1].value.as_char(), Some('-'));
		assert_eq!(operators[2].value.as_char(), Some('*'));
	}

	#[test]
	fn test_tokenize_instruction_line() {
		let options = test_options();
		let mut lexer = Lexer::new("LDA #$42", &options);
		let context = test_context();
		let tokens = lexer.tokenize(&context).unwrap();

		let significant: Vec<_> = tokens.iter().filter(|t| t.is_significant()).collect();

		assert_eq!(significant.len(), 3); // LDA, #, $42
		assert_eq!(significant[0].token_type, TokenType::Identifier);
		assert_eq!(significant[0].value.as_string(), Some("LDA"));
		assert_eq!(significant[1].token_type, TokenType::Operator);
		assert_eq!(significant[1].value.as_char(), Some('#'));
		assert_eq!(significant[2].token_type, TokenType::Number);
		assert_eq!(significant[2].value.as_number(), Some(0x42));
	}

	#[test]
	fn test_string_escapes() {
		let options = test_options();
		let mut lexer = Lexer::new("\"line\\nbreak\"", &options);
		let context = test_context();
		let tokens = lexer.tokenize(&context).unwrap();

		assert_eq!(tokens[0].token_type, TokenType::String);
		assert_eq!(tokens[0].value.as_string(), Some("line\nbreak"));
	}

	#[test]
	fn test_unterminated_string() {
		let options = test_options();
		let mut lexer = Lexer::new("\"unterminated", &options);
		let context = test_context();
		let result = lexer.tokenize(&context);

		assert!(result.is_err());
	}

	#[test]
	fn test_c_style_comments() {
		let mut options = test_options();
		options.allow_c_comments = true;
		let mut lexer = Lexer::new("// C-style comment", &options);
		let context = test_context();
		let tokens = lexer.tokenize(&context).unwrap();

		assert_eq!(tokens.len(), 2); // comment + EOF
		assert_eq!(tokens[0].token_type, TokenType::Comment);
	}
}
