//! Expression parsing and evaluation for the NES compiler
//!
//! This module handles parsing and evaluation of mathematical expressions,
//! symbol references, and function calls within assembly source code.

use crate::error::{AssemblyError, AssemblyResult, SourcePos};
use crate::parsing::tokens::{Token, TokenType};
use crate::symbols::{SymbolTable, SymbolValue};
use std::collections::HashMap;
use std::fmt;

/// Expression value types
#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionValue {
	/// Integer value
	Integer(i32),
	/// String value
	String(String),
	/// Boolean value
	Boolean(bool),
	/// Undefined (for forward references)
	Undefined,
}

impl ExpressionValue {
	/// Convert to integer if possible
	pub fn as_integer(&self) -> Option<i32> {
		match self {
			Self::Integer(val) => Some(*val),
			Self::Boolean(true) => Some(1),
			Self::Boolean(false) => Some(0),
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

	/// Convert to boolean if possible
	pub fn as_boolean(&self) -> Option<bool> {
		match self {
			Self::Boolean(b) => Some(*b),
			Self::Integer(0) => Some(false),
			Self::Integer(_) => Some(true),
			_ => None,
		}
	}

	/// Check if value is defined
	pub fn is_defined(&self) -> bool {
		!matches!(self, Self::Undefined)
	}

	/// Check if value is numeric
	pub fn is_numeric(&self) -> bool {
		matches!(self, Self::Integer(_) | Self::Boolean(_))
	}
}

impl fmt::Display for ExpressionValue {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Integer(val) => write!(f, "{}", val),
			Self::String(s) => write!(f, "\"{}\"", s),
			Self::Boolean(b) => write!(f, "{}", b),
			Self::Undefined => write!(f, "<undefined>"),
		}
	}
}

/// Expression AST node types
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
	/// Literal integer value
	Integer(i32),
	/// Literal string value
	String(String),
	/// Symbol reference
	Symbol(String),
	/// Binary operation
	Binary {
		left: Box<Expression>,
		operator: BinaryOp,
		right: Box<Expression>,
	},
	/// Unary operation
	Unary {
		operator: UnaryOp,
		operand: Box<Expression>,
	},
	/// Function call
	FunctionCall {
		name: String,
		args: Vec<Expression>,
	},
	/// Parenthesized expression
	Parentheses(Box<Expression>),
	/// Conditional expression (condition ? true_expr : false_expr)
	Conditional {
		condition: Box<Expression>,
		true_expr: Box<Expression>,
		false_expr: Box<Expression>,
	},
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
	// Arithmetic
	Add,
	Subtract,
	Multiply,
	Divide,
	Modulo,

	// Bitwise
	BitwiseAnd,
	BitwiseOr,
	BitwiseXor,
	LeftShift,
	RightShift,

	// Logical
	LogicalAnd,
	LogicalOr,

	// Comparison
	Equal,
	NotEqual,
	LessThan,
	LessThanOrEqual,
	GreaterThan,
	GreaterThanOrEqual,
}

impl BinaryOp {
	/// Get operator precedence (higher number = higher precedence)
	pub fn precedence(self) -> u8 {
		match self {
			Self::LogicalOr => 1,
			Self::LogicalAnd => 2,
			Self::BitwiseOr => 3,
			Self::BitwiseXor => 4,
			Self::BitwiseAnd => 5,
			Self::Equal | Self::NotEqual => 6,
			Self::LessThan
			| Self::LessThanOrEqual
			| Self::GreaterThan
			| Self::GreaterThanOrEqual => 7,
			Self::LeftShift | Self::RightShift => 8,
			Self::Add | Self::Subtract => 9,
			Self::Multiply | Self::Divide | Self::Modulo => 10,
		}
	}

	/// Check if operator is left-associative
	pub fn is_left_associative(self) -> bool {
		true // All our operators are left-associative
	}

	/// Parse operator from token
	pub fn from_token(token: &Token) -> Option<Self> {
		match &token.text[..] {
			"+" => Some(Self::Add),
			"-" => Some(Self::Subtract),
			"*" => Some(Self::Multiply),
			"/" => Some(Self::Divide),
			"%" => Some(Self::Modulo),
			"&" => Some(Self::BitwiseAnd),
			"|" => Some(Self::BitwiseOr),
			"^" => Some(Self::BitwiseXor),
			"<<" => Some(Self::LeftShift),
			">>" => Some(Self::RightShift),
			"&&" => Some(Self::LogicalAnd),
			"||" => Some(Self::LogicalOr),
			"==" => Some(Self::Equal),
			"!=" => Some(Self::NotEqual),
			"<" => Some(Self::LessThan),
			"<=" => Some(Self::LessThanOrEqual),
			">" => Some(Self::GreaterThan),
			">=" => Some(Self::GreaterThanOrEqual),
			_ => None,
		}
	}
}

impl fmt::Display for BinaryOp {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let op = match self {
			Self::Add => "+",
			Self::Subtract => "-",
			Self::Multiply => "*",
			Self::Divide => "/",
			Self::Modulo => "%",
			Self::BitwiseAnd => "&",
			Self::BitwiseOr => "|",
			Self::BitwiseXor => "^",
			Self::LeftShift => "<<",
			Self::RightShift => ">>",
			Self::LogicalAnd => "&&",
			Self::LogicalOr => "||",
			Self::Equal => "==",
			Self::NotEqual => "!=",
			Self::LessThan => "<",
			Self::LessThanOrEqual => "<=",
			Self::GreaterThan => ">",
			Self::GreaterThanOrEqual => ">=",
		};
		write!(f, "{}", op)
	}
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
	/// Unary plus
	Plus,
	/// Unary minus (negation)
	Minus,
	/// Logical NOT
	LogicalNot,
	/// Bitwise NOT
	BitwiseNot,
	/// LOW byte function
	Low,
	/// HIGH byte function
	High,
	/// BANK function
	Bank,
}

impl UnaryOp {
	/// Parse operator from token
	pub fn from_token(token: &Token) -> Option<Self> {
		match &token.text[..] {
			"+" => Some(Self::Plus),
			"-" => Some(Self::Minus),
			"!" => Some(Self::LogicalNot),
			"~" => Some(Self::BitwiseNot),
			"LOW" | "low" => Some(Self::Low),
			"HIGH" | "high" => Some(Self::High),
			"BANK" | "bank" => Some(Self::Bank),
			_ => None,
		}
	}
}

impl fmt::Display for UnaryOp {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let op = match self {
			Self::Plus => "+",
			Self::Minus => "-",
			Self::LogicalNot => "!",
			Self::BitwiseNot => "~",
			Self::Low => "LOW",
			Self::High => "HIGH",
			Self::Bank => "BANK",
		};
		write!(f, "{}", op)
	}
}

/// Expression parser using recursive descent
pub struct ExpressionParser {
	/// Tokens to parse
	tokens: Vec<Token>,
	/// Current token position
	position: usize,
	/// Current source position for error reporting
	current_pos: SourcePos,
}

impl ExpressionParser {
	/// Create a new expression parser
	pub fn new(tokens: Vec<Token>, pos: SourcePos) -> Self {
		Self {
			tokens,
			position: 0,
			current_pos: pos,
		}
	}

	/// Parse an expression from tokens
	pub fn parse(&mut self) -> AssemblyResult<Expression> {
		self.parse_conditional()
	}

	/// Parse conditional expression (ternary operator)
	fn parse_conditional(&mut self) -> AssemblyResult<Expression> {
		let condition = self.parse_logical_or()?;

		if self.consume_if_match("?") {
			let true_expr = Box::new(self.parse_expression()?);
			self.expect(":")?;
			let false_expr = Box::new(self.parse_conditional()?);

			Ok(Expression::Conditional {
				condition: Box::new(condition),
				true_expr,
				false_expr,
			})
		} else {
			Ok(condition)
		}
	}

	/// Parse logical OR expression
	fn parse_logical_or(&mut self) -> AssemblyResult<Expression> {
		self.parse_binary_left_associative(Self::parse_logical_and, &[BinaryOp::LogicalOr])
	}

	/// Parse logical AND expression
	fn parse_logical_and(&mut self) -> AssemblyResult<Expression> {
		self.parse_binary_left_associative(Self::parse_bitwise_or, &[BinaryOp::LogicalAnd])
	}

	/// Parse bitwise OR expression
	fn parse_bitwise_or(&mut self) -> AssemblyResult<Expression> {
		self.parse_binary_left_associative(Self::parse_bitwise_xor, &[BinaryOp::BitwiseOr])
	}

	/// Parse bitwise XOR expression
	fn parse_bitwise_xor(&mut self) -> AssemblyResult<Expression> {
		self.parse_binary_left_associative(Self::parse_bitwise_and, &[BinaryOp::BitwiseXor])
	}

	/// Parse bitwise AND expression
	fn parse_bitwise_and(&mut self) -> AssemblyResult<Expression> {
		self.parse_binary_left_associative(Self::parse_equality, &[BinaryOp::BitwiseAnd])
	}

	/// Parse equality expression
	fn parse_equality(&mut self) -> AssemblyResult<Expression> {
		self.parse_binary_left_associative(
			Self::parse_relational,
			&[BinaryOp::Equal, BinaryOp::NotEqual],
		)
	}

	/// Parse relational expression
	fn parse_relational(&mut self) -> AssemblyResult<Expression> {
		self.parse_binary_left_associative(
			Self::parse_shift,
			&[
				BinaryOp::LessThan,
				BinaryOp::LessThanOrEqual,
				BinaryOp::GreaterThan,
				BinaryOp::GreaterThanOrEqual,
			],
		)
	}

	/// Parse shift expression
	fn parse_shift(&mut self) -> AssemblyResult<Expression> {
		self.parse_binary_left_associative(
			Self::parse_additive,
			&[BinaryOp::LeftShift, BinaryOp::RightShift],
		)
	}

	/// Parse additive expression
	fn parse_additive(&mut self) -> AssemblyResult<Expression> {
		self.parse_binary_left_associative(
			Self::parse_multiplicative,
			&[BinaryOp::Add, BinaryOp::Subtract],
		)
	}

	/// Parse multiplicative expression
	fn parse_multiplicative(&mut self) -> AssemblyResult<Expression> {
		self.parse_binary_left_associative(
			Self::parse_unary,
			&[BinaryOp::Multiply, BinaryOp::Divide, BinaryOp::Modulo],
		)
	}

	/// Parse unary expression
	fn parse_unary(&mut self) -> AssemblyResult<Expression> {
		if let Some(op) = self.current_token().and_then(|t| UnaryOp::from_token(t)) {
			self.advance();
			let operand = Box::new(self.parse_unary()?);
			Ok(Expression::Unary {
				operator: op,
				operand,
			})
		} else {
			self.parse_primary()
		}
	}

	/// Parse primary expression
	fn parse_primary(&mut self) -> AssemblyResult<Expression> {
		let token = self.current_token().ok_or_else(|| {
			AssemblyError::parse(
				self.current_pos.clone(),
				"Unexpected end of expression".to_string(),
			)
		})?;

		match token.token_type {
			TokenType::Number => {
				let value = token.value.as_number().unwrap();
				self.advance();
				Ok(Expression::Integer(value))
			}
			TokenType::String => {
				let value = token.value.as_string().unwrap().to_string();
				self.advance();
				Ok(Expression::String(value))
			}
			TokenType::Identifier => {
				let name = token.value.as_string().unwrap().to_string();
				self.advance();

				// Check for function call
				if self.consume_if_match("(") {
					let mut args = Vec::new();

					if !self.check(")") {
						loop {
							args.push(self.parse_expression()?);
							if !self.consume_if_match(",") {
								break;
							}
						}
					}

					self.expect(")")?;
					Ok(Expression::FunctionCall {
						name,
						args,
					})
				} else {
					Ok(Expression::Symbol(name))
				}
			}
			TokenType::LeftParen => {
				self.advance(); // consume '('
				let expr = self.parse_expression()?;
				self.expect(")")?;
				Ok(Expression::Parentheses(Box::new(expr)))
			}
			_ => Err(AssemblyError::parse(
				self.current_pos.clone(),
				format!("Unexpected token in expression: {}", token.text),
			)),
		}
	}

	/// Parse expression (top-level entry point)
	fn parse_expression(&mut self) -> AssemblyResult<Expression> {
		self.parse_conditional()
	}

	/// Helper for parsing left-associative binary operations
	fn parse_binary_left_associative<F>(
		&mut self,
		mut parse_operand: F,
		operators: &[BinaryOp],
	) -> AssemblyResult<Expression>
	where
		F: FnMut(&mut Self) -> AssemblyResult<Expression>,
	{
		let mut left = parse_operand(self)?;

		while let Some(token) = self.current_token() {
			if let Some(op) = BinaryOp::from_token(token) {
				if operators.contains(&op) {
					self.advance();
					let right = Box::new(parse_operand(self)?);
					left = Expression::Binary {
						left: Box::new(left),
						operator: op,
						right,
					};
				} else {
					break;
				}
			} else {
				break;
			}
		}

		Ok(left)
	}

	/// Get current token
	fn current_token(&self) -> Option<&Token> {
		self.tokens.get(self.position)
	}

	/// Advance to next token
	fn advance(&mut self) {
		if self.position < self.tokens.len() {
			self.position += 1;
		}
	}

	/// Check if current token matches the given text
	fn check(&self, text: &str) -> bool {
		self.current_token().map_or(false, |t| t.text == text)
	}

	/// Consume token if it matches the given text
	fn consume_if_match(&mut self, text: &str) -> bool {
		if self.check(text) {
			self.advance();
			true
		} else {
			false
		}
	}

	/// Expect a specific token and consume it
	fn expect(&mut self, text: &str) -> AssemblyResult<()> {
		if self.consume_if_match(text) {
			Ok(())
		} else {
			let actual = self.current_token().map_or("EOF".to_string(), |t| t.text.clone());
			Err(AssemblyError::parse(
				self.current_pos.clone(),
				format!("Expected '{}' but found '{}'", text, actual),
			))
		}
	}
}

/// Expression evaluator
pub struct ExpressionEvaluator<'a> {
	/// Symbol table for symbol resolution
	symbol_table: Option<&'a SymbolTable>,
	/// Function definitions
	functions: HashMap<String, BuiltinFunction>,
}

/// Built-in function type
type BuiltinFunction = fn(&[ExpressionValue]) -> AssemblyResult<ExpressionValue>;

impl<'a> ExpressionEvaluator<'a> {
	/// Create a new expression evaluator
	pub fn new(symbol_table: Option<&'a SymbolTable>) -> Self {
		let mut functions = HashMap::new();

		// Register built-in functions
		functions.insert("LOW".to_string(), Self::builtin_low as BuiltinFunction);
		functions.insert("HIGH".to_string(), Self::builtin_high as BuiltinFunction);
		functions.insert("BANK".to_string(), Self::builtin_bank as BuiltinFunction);
		functions.insert("SIZEOF".to_string(), Self::builtin_sizeof as BuiltinFunction);

		Self {
			symbol_table,
			functions,
		}
	}

	/// Evaluate an expression
	pub fn evaluate(&self, expr: &Expression) -> AssemblyResult<ExpressionValue> {
		match expr {
			Expression::Integer(val) => Ok(ExpressionValue::Integer(*val)),
			Expression::String(s) => Ok(ExpressionValue::String(s.clone())),
			Expression::Symbol(name) => self.evaluate_symbol(name),
			Expression::Binary {
				left,
				operator,
				right,
			} => self.evaluate_binary(left, *operator, right),
			Expression::Unary {
				operator,
				operand,
			} => self.evaluate_unary(*operator, operand),
			Expression::FunctionCall {
				name,
				args,
			} => self.evaluate_function_call(name, args),
			Expression::Parentheses(expr) => self.evaluate(expr),
			Expression::Conditional {
				condition,
				true_expr,
				false_expr,
			} => self.evaluate_conditional(condition, true_expr, false_expr),
		}
	}

	/// Evaluate symbol reference
	fn evaluate_symbol(&self, name: &str) -> AssemblyResult<ExpressionValue> {
		if let Some(symbol_table) = self.symbol_table {
			if let Some(symbol) = symbol_table.get(name) {
				match symbol.value() {
					SymbolValue::Number(val) => Ok(ExpressionValue::Integer(*val)),
					SymbolValue::Address(addr) => Ok(ExpressionValue::Integer(*addr as i32)),
					SymbolValue::String(s) => Ok(ExpressionValue::String(s.clone())),
					SymbolValue::Undefined => Ok(ExpressionValue::Undefined),
					SymbolValue::Expression(_) => Ok(ExpressionValue::Undefined),
				}
			} else {
				Ok(ExpressionValue::Undefined)
			}
		} else {
			Ok(ExpressionValue::Undefined)
		}
	}

	/// Evaluate binary operation
	fn evaluate_binary(
		&self,
		left: &Expression,
		operator: BinaryOp,
		right: &Expression,
	) -> AssemblyResult<ExpressionValue> {
		let left_val = self.evaluate(left)?;
		let right_val = self.evaluate(right)?;

		match (left_val.as_integer(), right_val.as_integer()) {
			(Some(l), Some(r)) => {
				let result = match operator {
					BinaryOp::Add => l.wrapping_add(r),
					BinaryOp::Subtract => l.wrapping_sub(r),
					BinaryOp::Multiply => l.wrapping_mul(r),
					BinaryOp::Divide => {
						if r == 0 {
							return Err(AssemblyError::expression(
								SourcePos::file_only("expression".into()),
								"Division by zero".to_string(),
							));
						}
						l / r
					}
					BinaryOp::Modulo => {
						if r == 0 {
							return Err(AssemblyError::expression(
								SourcePos::file_only("expression".into()),
								"Modulo by zero".to_string(),
							));
						}
						l % r
					}
					BinaryOp::BitwiseAnd => l & r,
					BinaryOp::BitwiseOr => l | r,
					BinaryOp::BitwiseXor => l ^ r,
					BinaryOp::LeftShift => l << (r & 31), // Limit shift to avoid undefined behavior
					BinaryOp::RightShift => l >> (r & 31),
					BinaryOp::LogicalAnd => {
						if l != 0 && r != 0 {
							1
						} else {
							0
						}
					}
					BinaryOp::LogicalOr => {
						if l != 0 || r != 0 {
							1
						} else {
							0
						}
					}
					BinaryOp::Equal => {
						if l == r {
							1
						} else {
							0
						}
					}
					BinaryOp::NotEqual => {
						if l != r {
							1
						} else {
							0
						}
					}
					BinaryOp::LessThan => {
						if l < r {
							1
						} else {
							0
						}
					}
					BinaryOp::LessThanOrEqual => {
						if l <= r {
							1
						} else {
							0
						}
					}
					BinaryOp::GreaterThan => {
						if l > r {
							1
						} else {
							0
						}
					}
					BinaryOp::GreaterThanOrEqual => {
						if l >= r {
							1
						} else {
							0
						}
					}
				};
				Ok(ExpressionValue::Integer(result))
			}
			_ => Ok(ExpressionValue::Undefined),
		}
	}

	/// Evaluate unary operation
	fn evaluate_unary(
		&self,
		operator: UnaryOp,
		operand: &Expression,
	) -> AssemblyResult<ExpressionValue> {
		let operand_val = self.evaluate(operand)?;

		match operand_val.as_integer() {
			Some(val) => {
				let result = match operator {
					UnaryOp::Plus => val,
					UnaryOp::Minus => -val,
					UnaryOp::LogicalNot => {
						if val == 0 {
							1
						} else {
							0
						}
					}
					UnaryOp::BitwiseNot => !val,
					UnaryOp::Low => val & 0xFF,
					UnaryOp::High => (val >> 8) & 0xFF,
					UnaryOp::Bank => (val >> 13) & 0xFF, // Assuming 8KB banks
				};
				Ok(ExpressionValue::Integer(result))
			}
			None => Ok(ExpressionValue::Undefined),
		}
	}

	/// Evaluate function call
	fn evaluate_function_call(
		&self,
		name: &str,
		args: &[Expression],
	) -> AssemblyResult<ExpressionValue> {
		let arg_values: Result<Vec<_>, _> = args.iter().map(|arg| self.evaluate(arg)).collect();
		let arg_values = arg_values?;

		if let Some(func) = self.functions.get(name) {
			func(&arg_values)
		} else {
			Err(AssemblyError::expression(
				SourcePos::file_only("expression".into()),
				format!("Unknown function: {}", name),
			))
		}
	}

	/// Evaluate conditional expression
	fn evaluate_conditional(
		&self,
		condition: &Expression,
		true_expr: &Expression,
		false_expr: &Expression,
	) -> AssemblyResult<ExpressionValue> {
		let condition_val = self.evaluate(condition)?;

		if let Some(cond_bool) = condition_val.as_boolean() {
			if cond_bool {
				self.evaluate(true_expr)
			} else {
				self.evaluate(false_expr)
			}
		} else {
			Ok(ExpressionValue::Undefined)
		}
	}

	// Built-in functions
	fn builtin_low(args: &[ExpressionValue]) -> AssemblyResult<ExpressionValue> {
		if args.len() != 1 {
			return Err(AssemblyError::expression(
				SourcePos::file_only("expression".into()),
				"LOW function requires exactly 1 argument".to_string(),
			));
		}

		if let Some(val) = args[0].as_integer() {
			Ok(ExpressionValue::Integer(val & 0xFF))
		} else {
			Ok(ExpressionValue::Undefined)
		}
	}

	fn builtin_high(args: &[ExpressionValue]) -> AssemblyResult<ExpressionValue> {
		if args.len() != 1 {
			return Err(AssemblyError::expression(
				SourcePos::file_only("expression".into()),
				"HIGH function requires exactly 1 argument".to_string(),
			));
		}

		if let Some(val) = args[0].as_integer() {
			Ok(ExpressionValue::Integer((val >> 8) & 0xFF))
		} else {
			Ok(ExpressionValue::Undefined)
		}
	}

	fn builtin_bank(args: &[ExpressionValue]) -> AssemblyResult<ExpressionValue> {
		if args.len() != 1 {
			return Err(AssemblyError::expression(
				SourcePos::file_only("expression".into()),
				"BANK function requires exactly 1 argument".to_string(),
			));
		}

		if let Some(val) = args[0].as_integer() {
			Ok(ExpressionValue::Integer((val >> 13) & 0xFF))
		} else {
			Ok(ExpressionValue::Undefined)
		}
	}

	fn builtin_sizeof(args: &[ExpressionValue]) -> AssemblyResult<ExpressionValue> {
		if args.len() != 1 {
			return Err(AssemblyError::expression(
				SourcePos::file_only("expression".into()),
				"SIZEOF function requires exactly 1 argument".to_string(),
			));
		}

		// For now, return a placeholder size
		// In a real implementation, this would look up the symbol's size
		Ok(ExpressionValue::Integer(1))
	}
}

/// Convert lexer token to expression token
fn convert_lexer_token(lexer_token: crate::parsing::lexer::Token) -> Token {
	use crate::parsing::lexer;
	use crate::parsing::tokens;

	// Convert token type - lexer has simplified types, map to expression tokens based on content
	let token_type = match lexer_token.token_type {
		lexer::TokenType::Identifier => tokens::TokenType::Identifier,
		lexer::TokenType::Number => tokens::TokenType::Number,
		lexer::TokenType::String => tokens::TokenType::String,
		lexer::TokenType::Character => tokens::TokenType::Character,
		lexer::TokenType::Operator => {
			// Map specific operators based on text content
			match lexer_token.text.as_str() {
				"+" => tokens::TokenType::Plus,
				"-" => tokens::TokenType::Minus,
				"*" => tokens::TokenType::Multiply,
				"/" => tokens::TokenType::Divide,
				"%" => tokens::TokenType::Modulo,
				"&" => tokens::TokenType::BitwiseAnd,
				"|" => tokens::TokenType::BitwiseOr,
				"^" => tokens::TokenType::BitwiseXor,
				"~" => tokens::TokenType::BitwiseNot,
				"<<" => tokens::TokenType::LeftShift,
				">>" => tokens::TokenType::RightShift,
				"&&" => tokens::TokenType::LogicalAnd,
				"||" => tokens::TokenType::LogicalOr,
				"==" => tokens::TokenType::Equal,
				"!=" => tokens::TokenType::NotEqual,
				"<" => tokens::TokenType::LessThan,
				">" => tokens::TokenType::GreaterThan,
				"<=" => tokens::TokenType::LessEqual,
				">=" => tokens::TokenType::GreaterEqual,
				"(" => tokens::TokenType::LeftParen,
				")" => tokens::TokenType::RightParen,
				"[" => tokens::TokenType::LeftBracket,
				"]" => tokens::TokenType::RightBracket,
				"," => tokens::TokenType::Comma,
				":" => tokens::TokenType::Colon,
				";" => tokens::TokenType::Semicolon,
				"=" => tokens::TokenType::Assign,
				"?" => tokens::TokenType::Invalid, // No Question token available
				"." => tokens::TokenType::Invalid, // No Dot token available
				"#" => tokens::TokenType::Hash,
				"$" => tokens::TokenType::Dollar,
				"@" => tokens::TokenType::Invalid, // No At token available
				"!" => tokens::TokenType::LogicalNot,
				_ => tokens::TokenType::Invalid,
			}
		}
		lexer::TokenType::Comment => tokens::TokenType::Comment,
		lexer::TokenType::Whitespace => tokens::TokenType::Whitespace,
		lexer::TokenType::EndOfLine => tokens::TokenType::EndOfLine,
		lexer::TokenType::EndOfFile => tokens::TokenType::EndOfFile,
	};

	// Convert token value
	let token_value = match lexer_token.value {
		lexer::TokenValue::String(s) => tokens::TokenValue::String(s),
		lexer::TokenValue::Number(n) => tokens::TokenValue::Integer(n as i64),
		lexer::TokenValue::Character(c) => tokens::TokenValue::Character(c),
		lexer::TokenValue::Operator(c) => tokens::TokenValue::Character(c),
		lexer::TokenValue::None => tokens::TokenValue::None,
	};

	Token::new(
		token_type,
		token_value,
		lexer_token.text,
		lexer_token.pos.line,
		lexer_token.pos.column,
	)
}

/// Parse an expression from a string
pub fn parse_expression(input: &str, pos: SourcePos) -> AssemblyResult<Expression> {
	use crate::parsing::lexer::Lexer;
	use crate::parsing::{ParseContext, ParseOptions};

	let options = ParseOptions::default();
	let mut lexer = Lexer::new(input, &options);
	let mut context = ParseContext::new("<expression>".to_string());
	let lexer_tokens = lexer.tokenize(&context)?;

	// Filter out whitespace and comments and convert to tokens::Token
	let tokens: Vec<_> = lexer_tokens
		.into_iter()
		.filter(|t| t.is_significant())
		.map(|t| convert_lexer_token(t))
		.collect();

	let mut parser = ExpressionParser::new(tokens, pos);
	parser.parse()
}

/// Evaluate an expression string with optional symbol table
pub fn evaluate_expression(
	input: &str,
	pos: SourcePos,
	symbol_table: Option<&SymbolTable>,
) -> AssemblyResult<ExpressionValue> {
	let expr = parse_expression(input, pos)?;
	let evaluator = ExpressionEvaluator::new(symbol_table);
	evaluator.evaluate(&expr)
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::path::PathBuf;

	fn test_pos() -> SourcePos {
		SourcePos::new(PathBuf::from("test.asm"), 1, 1)
	}

	#[test]
	fn test_expression_value() {
		let int_val = ExpressionValue::Integer(42);
		assert_eq!(int_val.as_integer(), Some(42));
		assert!(int_val.is_numeric());
		assert!(int_val.is_defined());

		let undef_val = ExpressionValue::Undefined;
		assert!(!undef_val.is_defined());
		assert_eq!(undef_val.as_integer(), None);
	}

	#[test]
	fn test_binary_op_precedence() {
		assert!(BinaryOp::Multiply.precedence() > BinaryOp::Add.precedence());
		assert!(BinaryOp::Add.precedence() > BinaryOp::LogicalOr.precedence());
	}

	#[test]
	fn test_parse_simple_expressions() {
		let pos = test_pos();

		// Integer literal
		let expr = parse_expression("42", pos.clone()).unwrap();
		assert_eq!(expr, Expression::Integer(42));

		// Binary operation
		let expr = parse_expression("2 + 3", pos.clone()).unwrap();
		match expr {
			Expression::Binary {
				operator,
				..
			} => assert_eq!(operator, BinaryOp::Add),
			_ => panic!("Expected binary expression"),
		}

		// Parentheses
		let expr = parse_expression("(42)", pos).unwrap();
		match expr {
			Expression::Parentheses(inner) => match *inner {
				Expression::Integer(42) => {}
				_ => panic!("Expected integer in parentheses"),
			},
			_ => panic!("Expected parenthesized expression"),
		}
	}

	#[test]
	fn test_evaluate_expressions() {
		let pos = test_pos();
		let evaluator = ExpressionEvaluator::new(None);

		// Simple arithmetic
		let expr = parse_expression("2 + 3", pos.clone()).unwrap();
		let result = evaluator.evaluate(&expr).unwrap();
		assert_eq!(result.as_integer(), Some(5));

		// Precedence
		let expr = parse_expression("2 + 3 * 4", pos.clone()).unwrap();
		let result = evaluator.evaluate(&expr).unwrap();
		assert_eq!(result.as_integer(), Some(14)); // 2 + (3 * 4)

		// Bitwise operations
		let expr = parse_expression("0xFF & 0x0F", pos).unwrap();
		let result = evaluator.evaluate(&expr).unwrap();
		assert_eq!(result.as_integer(), Some(0x0F));
	}

	#[test]
	fn test_unary_operations() {
		let pos = test_pos();
		let evaluator = ExpressionEvaluator::new(None);

		// Negation
		let expr = parse_expression("-42", pos.clone()).unwrap();
		let result = evaluator.evaluate(&expr).unwrap();
		assert_eq!(result.as_integer(), Some(-42));

		// LOW/HIGH functions
		let expr = parse_expression("LOW(0x1234)", pos.clone()).unwrap();
		let result = evaluator.evaluate(&expr).unwrap();
		assert_eq!(result.as_integer(), Some(0x34));

		let expr = parse_expression("HIGH(0x1234)", pos).unwrap();
		let result = evaluator.evaluate(&expr).unwrap();
		assert_eq!(result.as_integer(), Some(0x12));
	}

	#[test]
	fn test_function_calls() {
		let pos = test_pos();
		let evaluator = ExpressionEvaluator::new(None);

		// Built-in function call
		let expr = parse_expression("LOW(0x1234)", pos).unwrap();
		match &expr {
			Expression::FunctionCall {
				name,
				args,
			} => {
				assert_eq!(name, "LOW");
				assert_eq!(args.len(), 1);
			}
			_ => panic!("Expected function call"),
		}

		let result = evaluator.evaluate(&expr).unwrap();
		assert_eq!(result.as_integer(), Some(0x34));
	}

	#[test]
	fn test_conditional_expressions() {
		let pos = test_pos();
		let evaluator = ExpressionEvaluator::new(None);

		// True condition
		let expr = parse_expression("1 ? 42 : 24", pos.clone()).unwrap();
		let result = evaluator.evaluate(&expr).unwrap();
		assert_eq!(result.as_integer(), Some(42));

		// False condition
		let expr = parse_expression("0 ? 42 : 24", pos).unwrap();
		let result = evaluator.evaluate(&expr).unwrap();
		assert_eq!(result.as_integer(), Some(24));
	}

	#[test]
	fn test_error_handling() {
		let pos = test_pos();
		let evaluator = ExpressionEvaluator::new(None);

		// Division by zero
		let expr = parse_expression("1 / 0", pos.clone()).unwrap();
		let result = evaluator.evaluate(&expr);
		assert!(result.is_err());

		// Invalid expression
		let result = parse_expression("2 +", pos);
		assert!(result.is_err());
	}
}
