//! Token definitions for the NES assembler lexer.
//!
//! This module defines all token types used during lexical analysis of
//! 6502 assembly source code.

use std::fmt;

/// Token types for assembly language lexical analysis
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
	// Literals
	/// Numeric literal (e.g., 42, $FF, %10101010)
	Number,
	/// String literal (e.g., "Hello, world!")
	String,
	/// Character literal (e.g., 'A')
	Character,

	// Identifiers and symbols
	/// Identifier (labels, symbols, etc.)
	Identifier,
	/// Local label (e.g., .loop)
	LocalLabel,
	/// Anonymous label (e.g., +, -)
	AnonymousLabel,

	// Operators
	/// Addition operator (+)
	Plus,
	/// Subtraction operator (-)
	Minus,
	/// Multiplication operator (*)
	Multiply,
	/// Division operator (/)
	Divide,
	/// Modulo operator (%)
	Modulo,
	/// Bitwise AND operator (&)
	BitwiseAnd,
	/// Bitwise OR operator (|)
	BitwiseOr,
	/// Bitwise XOR operator (^)
	BitwiseXor,
	/// Bitwise NOT operator (~)
	BitwiseNot,
	/// Left shift operator (<<)
	LeftShift,
	/// Right shift operator (>>)
	RightShift,
	/// Logical AND operator (&&)
	LogicalAnd,
	/// Logical OR operator (||)
	LogicalOr,
	/// Logical NOT operator (!)
	LogicalNot,
	/// Equality operator (==)
	Equal,
	/// Inequality operator (!=)
	NotEqual,
	/// Less than operator (<)
	LessThan,
	/// Less than or equal operator (<=)
	LessEqual,
	/// Greater than operator (>)
	GreaterThan,
	/// Greater than or equal operator (>=)
	GreaterEqual,

	// Punctuation
	/// Comma (,)
	Comma,
	/// Colon (:)
	Colon,
	/// Semicolon (;)
	Semicolon,
	/// Hash/immediate prefix (#)
	Hash,
	/// Dollar sign/hex prefix ($)
	Dollar,
	/// Percent sign/binary prefix (%)
	Percent,
	/// Left parenthesis (()
	LeftParen,
	/// Right parenthesis ())
	RightParen,
	/// Left bracket ([)
	LeftBracket,
	/// Right bracket (])
	RightBracket,
	/// Left brace ({)
	LeftBrace,
	/// Right brace (})
	RightBrace,
	/// Assignment operator (=)
	Assign,

	// Special tokens
	/// End of line
	EndOfLine,
	/// End of file
	EndOfFile,
	/// Comment (;...)
	Comment,
	/// Whitespace
	Whitespace,

	// Assembly-specific
	/// 6502 instruction mnemonic
	Instruction,
	/// Assembler directive
	Directive,
	/// Register name (A, X, Y)
	Register,

	// Addressing mode indicators
	/// Immediate addressing (#)
	Immediate,
	/// Zero page addressing
	ZeroPage,
	/// Absolute addressing
	Absolute,
	/// Indexed addressing (,X or ,Y)
	Indexed,
	/// Indirect addressing ((...))
	Indirect,

	// Error token
	/// Invalid or unrecognized token
	Invalid,
}

impl fmt::Display for TokenType {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let name = match self {
			Self::Number => "number",
			Self::String => "string",
			Self::Character => "character",
			Self::Identifier => "identifier",
			Self::LocalLabel => "local label",
			Self::AnonymousLabel => "anonymous label",
			Self::Plus => "+",
			Self::Minus => "-",
			Self::Multiply => "*",
			Self::Divide => "/",
			Self::Modulo => "%",
			Self::BitwiseAnd => "&",
			Self::BitwiseOr => "|",
			Self::BitwiseXor => "^",
			Self::BitwiseNot => "~",
			Self::LeftShift => "<<",
			Self::RightShift => ">>",
			Self::LogicalAnd => "&&",
			Self::LogicalOr => "||",
			Self::LogicalNot => "!",
			Self::Equal => "==",
			Self::NotEqual => "!=",
			Self::LessThan => "<",
			Self::LessEqual => "<=",
			Self::GreaterThan => ">",
			Self::GreaterEqual => ">=",
			Self::Comma => ",",
			Self::Colon => ":",
			Self::Semicolon => ";",
			Self::Hash => "#",
			Self::Dollar => "$",
			Self::Percent => "%",
			Self::LeftParen => "(",
			Self::RightParen => ")",
			Self::LeftBracket => "[",
			Self::RightBracket => "]",
			Self::LeftBrace => "{",
			Self::RightBrace => "}",
			Self::Assign => "=",
			Self::EndOfLine => "end of line",
			Self::EndOfFile => "end of file",
			Self::Comment => "comment",
			Self::Whitespace => "whitespace",
			Self::Instruction => "instruction",
			Self::Directive => "directive",
			Self::Register => "register",
			Self::Immediate => "immediate",
			Self::ZeroPage => "zero page",
			Self::Absolute => "absolute",
			Self::Indexed => "indexed",
			Self::Indirect => "indirect",
			Self::Invalid => "invalid",
		};
		write!(f, "{}", name)
	}
}

/// Token value types
#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue {
	/// No value
	None,
	/// Integer value
	Integer(i64),
	/// Floating point value
	Float(f64),
	/// String value
	String(String),
	/// Character value
	Character(char),
	/// Boolean value
	Boolean(bool),
}

impl TokenValue {
	/// Convert to number if possible
	pub fn as_number(&self) -> Option<i32> {
		match self {
			Self::Integer(i) => Some(*i as i32),
			Self::Float(f) => Some(*f as i32),
			Self::Character(c) => Some(*c as i32),
			_ => None,
		}
	}

	/// Convert to string if possible
	pub fn as_string(&self) -> Option<&str> {
		match self {
			Self::String(s) => Some(s),
			_ => None,
		}
	}
}

impl fmt::Display for TokenValue {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::None => write!(f, ""),
			Self::Integer(i) => write!(f, "{}", i),
			Self::Float(fl) => write!(f, "{}", fl),
			Self::String(s) => write!(f, "\"{}\"", s),
			Self::Character(c) => write!(f, "'{}'", c),
			Self::Boolean(b) => write!(f, "{}", b),
		}
	}
}

/// A complete token with type, value, and position information
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
	/// Token type
	pub token_type: TokenType,
	/// Token value (if applicable)
	pub value: TokenValue,
	/// Raw text of the token
	pub text: String,
	/// Line number (1-based)
	pub line: usize,
	/// Column number (1-based)
	pub column: usize,
	/// Length in characters
	pub length: usize,
}

impl Token {
	/// Create a new token
	pub fn new(
		token_type: TokenType,
		value: TokenValue,
		text: String,
		line: usize,
		column: usize,
	) -> Self {
		let length = text.chars().count();
		Self {
			token_type,
			value,
			text,
			line,
			column,
			length,
		}
	}

	/// Create a simple token with no value
	pub fn simple(token_type: TokenType, text: String, line: usize, column: usize) -> Self {
		Self::new(token_type, TokenValue::None, text, line, column)
	}

	/// Create a number token
	pub fn number(value: i64, text: String, line: usize, column: usize) -> Self {
		Self::new(TokenType::Number, TokenValue::Integer(value), text, line, column)
	}

	/// Create a string token
	pub fn string(value: String, text: String, line: usize, column: usize) -> Self {
		Self::new(TokenType::String, TokenValue::String(value), text, line, column)
	}

	/// Create an identifier token
	pub fn identifier(name: String, line: usize, column: usize) -> Self {
		Self::new(TokenType::Identifier, TokenValue::String(name.clone()), name, line, column)
	}

	/// Create an instruction token
	pub fn instruction(mnemonic: String, line: usize, column: usize) -> Self {
		Self::new(
			TokenType::Instruction,
			TokenValue::String(mnemonic.clone()),
			mnemonic,
			line,
			column,
		)
	}

	/// Create a directive token
	pub fn directive(name: String, line: usize, column: usize) -> Self {
		Self::new(TokenType::Directive, TokenValue::String(name.clone()), name, line, column)
	}

	/// Create an end of line token
	pub fn end_of_line(line: usize, column: usize) -> Self {
		Self::simple(TokenType::EndOfLine, "\n".to_string(), line, column)
	}

	/// Create an end of file token
	pub fn end_of_file(line: usize, column: usize) -> Self {
		Self::simple(TokenType::EndOfFile, "".to_string(), line, column)
	}

	/// Create an invalid token
	pub fn invalid(text: String, line: usize, column: usize) -> Self {
		Self::simple(TokenType::Invalid, text, line, column)
	}

	/// Check if this token is a specific type
	pub fn is_type(&self, token_type: TokenType) -> bool {
		self.token_type == token_type
	}

	/// Check if this token is whitespace
	pub fn is_whitespace(&self) -> bool {
		self.token_type == TokenType::Whitespace
	}

	/// Check if this token is a comment
	pub fn is_comment(&self) -> bool {
		self.token_type == TokenType::Comment
	}

	/// Check if this token should be ignored during parsing
	pub fn is_ignorable(&self) -> bool {
		matches!(self.token_type, TokenType::Whitespace | TokenType::Comment)
	}

	/// Check if this token is an operator
	pub fn is_operator(&self) -> bool {
		matches!(
			self.token_type,
			TokenType::Plus
				| TokenType::Minus
				| TokenType::Multiply
				| TokenType::Divide
				| TokenType::Modulo
				| TokenType::BitwiseAnd
				| TokenType::BitwiseOr
				| TokenType::BitwiseXor
				| TokenType::BitwiseNot
				| TokenType::LeftShift
				| TokenType::RightShift
				| TokenType::LogicalAnd
				| TokenType::LogicalOr
				| TokenType::LogicalNot
				| TokenType::Equal
				| TokenType::NotEqual
				| TokenType::LessThan
				| TokenType::LessEqual
				| TokenType::GreaterThan
				| TokenType::GreaterEqual
		)
	}

	/// Check if this token is a binary operator
	pub fn is_binary_operator(&self) -> bool {
		matches!(
			self.token_type,
			TokenType::Plus
				| TokenType::Minus
				| TokenType::Multiply
				| TokenType::Divide
				| TokenType::Modulo
				| TokenType::BitwiseAnd
				| TokenType::BitwiseOr
				| TokenType::BitwiseXor
				| TokenType::LeftShift
				| TokenType::RightShift
				| TokenType::LogicalAnd
				| TokenType::LogicalOr
				| TokenType::Equal
				| TokenType::NotEqual
				| TokenType::LessThan
				| TokenType::LessEqual
				| TokenType::GreaterThan
				| TokenType::GreaterEqual
		)
	}

	/// Check if this token is a unary operator
	pub fn is_unary_operator(&self) -> bool {
		matches!(
			self.token_type,
			TokenType::Plus | TokenType::Minus | TokenType::BitwiseNot | TokenType::LogicalNot
		)
	}

	/// Check if this token is a literal value
	pub fn is_literal(&self) -> bool {
		matches!(self.token_type, TokenType::Number | TokenType::String | TokenType::Character)
	}

	/// Get operator precedence (higher number = higher precedence)
	pub fn precedence(&self) -> u8 {
		match self.token_type {
			TokenType::LogicalOr => 1,
			TokenType::LogicalAnd => 2,
			TokenType::BitwiseOr => 3,
			TokenType::BitwiseXor => 4,
			TokenType::BitwiseAnd => 5,
			TokenType::Equal | TokenType::NotEqual => 6,
			TokenType::LessThan
			| TokenType::LessEqual
			| TokenType::GreaterThan
			| TokenType::GreaterEqual => 7,
			TokenType::LeftShift | TokenType::RightShift => 8,
			TokenType::Plus | TokenType::Minus => 9,
			TokenType::Multiply | TokenType::Divide | TokenType::Modulo => 10,
			// Unary operators have higher precedence
			TokenType::LogicalNot | TokenType::BitwiseNot => 11,
			_ => 0,
		}
	}

	/// Check if this operator is left-associative
	pub fn is_left_associative(&self) -> bool {
		// Most operators are left-associative
		// Right-associative operators would be things like assignment (=)
		!matches!(self.token_type, TokenType::Assign)
	}

	/// Get the string value if this is a string-valued token
	pub fn string_value(&self) -> Option<&str> {
		match &self.value {
			TokenValue::String(s) => Some(s),
			_ => None,
		}
	}

	/// Get the integer value if this is a numeric token
	pub fn integer_value(&self) -> Option<i64> {
		match self.value {
			TokenValue::Integer(i) => Some(i),
			_ => None,
		}
	}

	/// Get the character value if this is a character token
	pub fn character_value(&self) -> Option<char> {
		match self.value {
			TokenValue::Character(c) => Some(c),
			_ => None,
		}
	}

	/// Get end position (line, column)
	pub fn end_position(&self) -> (usize, usize) {
		(self.line, self.column + self.length)
	}
}

impl fmt::Display for Token {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{} '{}' at {}:{}", self.token_type, self.text, self.line, self.column)
	}
}

/// Token stream for parsing
#[derive(Debug, Clone)]
pub struct TokenStream {
	/// All tokens
	tokens: Vec<Token>,
	/// Current position
	position: usize,
}

impl TokenStream {
	/// Create a new token stream
	pub fn new(tokens: Vec<Token>) -> Self {
		Self {
			tokens,
			position: 0,
		}
	}

	/// Get current token without advancing
	pub fn peek(&self) -> Option<&Token> {
		self.tokens.get(self.position)
	}

	/// Get token at offset from current position
	pub fn peek_ahead(&self, offset: usize) -> Option<&Token> {
		self.tokens.get(self.position + offset)
	}

	/// Get current token and advance position
	pub fn next(&mut self) -> Option<&Token> {
		if self.position < self.tokens.len() {
			let token = &self.tokens[self.position];
			self.position += 1;
			Some(token)
		} else {
			None
		}
	}

	/// Get current token and advance, skipping ignorable tokens
	pub fn next_significant(&mut self) -> Option<Token> {
		while let Some(token) = self.next() {
			if !token.is_ignorable() {
				return Some(token.clone());
			}
		}
		None
	}

	/// Check if at end of stream
	pub fn is_at_end(&self) -> bool {
		self.position >= self.tokens.len()
	}

	/// Get current position
	pub fn position(&self) -> usize {
		self.position
	}

	/// Set position
	pub fn set_position(&mut self, position: usize) {
		self.position = position.min(self.tokens.len());
	}

	/// Save current position
	pub fn save_position(&self) -> usize {
		self.position
	}

	/// Restore saved position
	pub fn restore_position(&mut self, saved_position: usize) {
		self.set_position(saved_position);
	}

	/// Get remaining token count
	pub fn remaining(&self) -> usize {
		self.tokens.len().saturating_sub(self.position)
	}

	/// Get all tokens
	pub fn tokens(&self) -> &[Token] {
		&self.tokens
	}

	/// Skip ignorable tokens
	pub fn skip_ignorable(&mut self) {
		while let Some(token) = self.peek() {
			if token.is_ignorable() {
				self.position += 1;
			} else {
				break;
			}
		}
	}

	/// Expect a specific token type
	pub fn expect(&mut self, expected_type: TokenType) -> Option<Token> {
		self.skip_ignorable();
		if let Some(token) = self.peek() {
			if token.token_type == expected_type {
				let result = token.clone();
				self.position += 1;
				Some(result)
			} else {
				None
			}
		} else {
			None
		}
	}

	/// Check if next significant token matches type
	pub fn matches(&mut self, token_type: TokenType) -> bool {
		self.skip_ignorable();
		if let Some(token) = self.peek() {
			token.token_type == token_type
		} else {
			false
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_token_creation() {
		let token = Token::number(42, "$2A".to_string(), 1, 5);
		assert_eq!(token.token_type, TokenType::Number);
		assert_eq!(token.integer_value(), Some(42));
		assert_eq!(token.text, "$2A");
		assert_eq!(token.line, 1);
		assert_eq!(token.column, 5);
		assert_eq!(token.length, 3);
	}

	#[test]
	fn test_token_predicates() {
		let number = Token::number(42, "42".to_string(), 1, 1);
		assert!(number.is_literal());
		assert!(!number.is_operator());

		let plus = Token::simple(TokenType::Plus, "+".to_string(), 1, 1);
		assert!(plus.is_operator());
		assert!(plus.is_binary_operator());
		assert!(plus.is_unary_operator()); // + can be both
		assert!(!plus.is_literal());

		let minus = Token::simple(TokenType::Minus, "-".to_string(), 1, 1);
		assert!(minus.is_unary_operator());
		assert!(minus.is_binary_operator());
	}

	#[test]
	fn test_operator_precedence() {
		let plus = Token::simple(TokenType::Plus, "+".to_string(), 1, 1);
		let multiply = Token::simple(TokenType::Multiply, "*".to_string(), 1, 1);
		let logical_and = Token::simple(TokenType::LogicalAnd, "&&".to_string(), 1, 1);

		assert!(multiply.precedence() > plus.precedence());
		assert!(plus.precedence() > logical_and.precedence());
	}

	#[test]
	fn test_token_stream() {
		let tokens = vec![
			Token::identifier("test".to_string(), 1, 1),
			Token::simple(TokenType::Colon, ":".to_string(), 1, 5),
			Token::instruction("LDA".to_string(), 1, 7),
			Token::simple(TokenType::Hash, "#".to_string(), 1, 11),
			Token::number(42, "42".to_string(), 1, 12),
		];

		let mut stream = TokenStream::new(tokens);

		assert_eq!(stream.remaining(), 5);
		assert!(!stream.is_at_end());

		let first = stream.next().unwrap();
		assert_eq!(first.token_type, TokenType::Identifier);
		assert_eq!(first.string_value(), Some("test"));

		let second = stream.peek().unwrap();
		assert_eq!(second.token_type, TokenType::Colon);

		assert_eq!(stream.position(), 1);
		assert_eq!(stream.remaining(), 4);
	}

	#[test]
	fn test_token_stream_expect() {
		let tokens = vec![
			Token::identifier("label".to_string(), 1, 1),
			Token::simple(TokenType::Colon, ":".to_string(), 1, 6),
		];

		let mut stream = TokenStream::new(tokens);

		assert!(stream.expect(TokenType::Identifier).is_some());
		assert!(stream.expect(TokenType::Colon).is_some());
		assert!(stream.expect(TokenType::Instruction).is_none());
		assert!(stream.is_at_end());
	}

	#[test]
	fn test_token_stream_save_restore() {
		let tokens = vec![
			Token::instruction("LDA".to_string(), 1, 1),
			Token::number(42, "42".to_string(), 1, 5),
		];

		let mut stream = TokenStream::new(tokens);
		let saved = stream.save_position();

		stream.next();
		stream.next();
		assert!(stream.is_at_end());

		stream.restore_position(saved);
		assert!(!stream.is_at_end());
		assert_eq!(stream.position(), 0);
	}

	#[test]
	fn test_token_value_display() {
		assert_eq!(format!("{}", TokenValue::None), "");
		assert_eq!(format!("{}", TokenValue::Integer(42)), "42");
		assert_eq!(format!("{}", TokenValue::String("test".to_string())), "\"test\"");
		assert_eq!(format!("{}", TokenValue::Character('A')), "'A'");
		assert_eq!(format!("{}", TokenValue::Boolean(true)), "true");
	}

	#[test]
	fn test_token_display() {
		let token = Token::number(42, "$2A".to_string(), 5, 10);
		let display = format!("{}", token);
		assert!(display.contains("number"));
		assert!(display.contains("$2A"));
		assert!(display.contains("5:10"));
	}

	#[test]
	fn test_token_end_position() {
		let token = Token::identifier("hello".to_string(), 1, 5);
		let (end_line, end_col) = token.end_position();
		assert_eq!(end_line, 1);
		assert_eq!(end_col, 10); // 5 + 5 characters
	}

	#[test]
	fn test_token_type_display() {
		assert_eq!(format!("{}", TokenType::Number), "number");
		assert_eq!(format!("{}", TokenType::Plus), "+");
		assert_eq!(format!("{}", TokenType::Identifier), "identifier");
		assert_eq!(format!("{}", TokenType::EndOfFile), "end of file");
	}
}
