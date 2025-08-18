//! Error handling for the NES assembler.
//!
//! This module provides comprehensive error types and utilities for reporting
//! assembly errors with detailed context information.

use std::fmt;
use std::num::{ParseFloatError, ParseIntError};
use std::path::PathBuf;
use thiserror::Error;

/// Result type for assembly operations.
pub type AssemblyResult<T> = Result<T, AssemblyError>;

/// Represents a position in source code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePos {
	/// File path
	pub file: PathBuf,
	/// Line number (1-based)
	pub line: usize,
	/// Column number (1-based)
	pub column: usize,
}

impl SourcePos {
	/// Create a new source position.
	pub fn new(file: PathBuf, line: usize, column: usize) -> Self {
		Self {
			file,
			line,
			column,
		}
	}

	/// Create a new source position with just a file.
	pub fn file_only(file: PathBuf) -> Self {
		Self {
			file,
			line: 1,
			column: 1,
		}
	}
}

impl fmt::Display for SourcePos {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}:{}:{}", self.file.display(), self.line, self.column)
	}
}

/// Comprehensive error type for assembly operations.
#[derive(Error, Debug)]
pub enum AssemblyError {
	/// I/O related errors
	#[error("I/O error{}: {source}", pos.as_ref().map(|p| format!(" at {}", p)).unwrap_or_default())]
	Io {
		pos: Option<SourcePos>,
		#[source]
		source: std::io::Error,
	},

	/// Parse errors for instructions or directives
	#[error("Parse error at {pos}: {message}")]
	Parse {
		pos: SourcePos,
		message: String,
	},

	/// Invalid instruction or addressing mode
	#[error("Invalid instruction at {pos}: {message}")]
	InvalidInstruction {
		pos: SourcePos,
		message: String,
	},

	/// Symbol-related errors
	#[error("Symbol error at {pos}: {message}")]
	Symbol {
		pos: SourcePos,
		message: String,
	},

	/// Undefined symbol reference
	#[error("Undefined symbol '{symbol}' at {pos}")]
	UndefinedSymbol {
		pos: SourcePos,
		symbol: String,
	},

	/// Duplicate symbol definition
	#[error("Duplicate symbol '{symbol}' at {pos} (previously defined at {previous_pos})")]
	DuplicateSymbol {
		pos: SourcePos,
		symbol: String,
		previous_pos: SourcePos,
	},

	/// Memory management errors
	#[error("Memory error at {pos}: {message}")]
	Memory {
		pos: SourcePos,
		message: String,
	},

	/// Bank overflow
	#[error(
		"Bank overflow at {pos}: trying to write {size} bytes but only {available} bytes available"
	)]
	BankOverflow {
		pos: SourcePos,
		size: usize,
		available: usize,
	},

	/// Invalid memory address
	#[error("Invalid memory address {address:#06X} at {pos}: {message}")]
	InvalidAddress {
		pos: SourcePos,
		address: u16,
		message: String,
	},

	/// Macro-related errors
	#[error("Macro error at {pos}: {message}")]
	Macro {
		pos: SourcePos,
		message: String,
	},

	/// Expression evaluation errors
	#[error("Expression error at {pos}: {message}")]
	Expression {
		pos: SourcePos,
		message: String,
	},

	/// Numeric conversion errors
	#[error("Number parsing error at {pos}: {message}")]
	NumberParse {
		pos: SourcePos,
		message: String,
		#[source]
		source: Option<Box<dyn std::error::Error + Send + Sync>>,
	},

	/// Output format errors
	#[error("Output error{}: {message}", pos.as_ref().map(|p| format!(" at {}", p)).unwrap_or_default())]
	Output {
		pos: Option<SourcePos>,
		message: String,
	},

	/// Configuration errors
	#[error("Configuration error: {message}")]
	Config {
		message: String,
	},

	/// Platform-specific errors (e.g., NES-specific validation)
	#[error("Platform error at {pos}: {message}")]
	Platform {
		pos: SourcePos,
		message: String,
	},

	/// Internal compiler errors (should not happen in normal operation)
	#[error("Internal error{}: {message}", pos.as_ref().map(|p| format!(" at {}", p)).unwrap_or_default())]
	Internal {
		pos: Option<SourcePos>,
		message: String,
	},

	/// Multiple errors collected during assembly
	#[error("Multiple errors occurred during assembly")]
	Multiple {
		errors: Vec<AssemblyError>,
	},

	/// First pass errors that prevent continuation
	#[error("First pass failed with {count} errors")]
	FirstPassErrors {
		count: usize,
	},

	/// Invalid configuration parameter
	#[error("Invalid configuration: {parameter}")]
	InvalidConfiguration {
		parameter: String,
	},

	/// Parameter errors in macros
	#[error("Parameter error at {pos}: {message}")]
	ParameterError {
		pos: SourcePos,
		message: String,
	},

	/// Expansion errors in macros
	#[error("Expansion error at {pos}: {message}")]
	ExpansionError {
		pos: SourcePos,
		message: String,
	},

	/// Invalid definition errors
	#[error("Invalid definition at {pos}: {message}")]
	InvalidDefinition {
		pos: SourcePos,
		message: String,
	},

	/// Not found errors
	#[error("Not found at {pos}: {message}")]
	NotFound {
		pos: SourcePos,
		message: String,
	},

	/// Invalid directive parameter
	#[error("Invalid directive parameter at {pos}: {message}")]
	InvalidDirectiveParameter {
		pos: SourcePos,
		message: String,
	},

	/// Invalid iNES parameter
	#[error("Invalid iNES parameter at {pos}: {message}")]
	InvalidINesParameter {
		pos: SourcePos,
		message: String,
	},

	/// Invalid mapper
	#[error("Invalid mapper at {pos}: {message}")]
	InvalidMapper {
		pos: SourcePos,
		message: String,
	},

	/// Already exists error
	#[error("Already exists: {name}")]
	AlreadyExists {
		name: String,
	},

	/// Scope error
	#[error("Scope error: {message}")]
	ScopeError {
		message: String,
	},

	/// Macro-related errors
	#[error("Macro error at {pos}: {message}")]
	MacroError {
		pos: SourcePos,
		message: String,
	},
}

/// Type alias for macro errors (for compatibility)
pub type MacroError = AssemblyError;

impl Clone for AssemblyError {
	fn clone(&self) -> Self {
		match self {
			Self::Io {
				pos,
				source,
			} => Self::Io {
				pos: pos.clone(),
				source: std::io::Error::new(source.kind(), source.to_string()),
			},
			Self::Parse {
				pos,
				message,
			} => Self::Parse {
				pos: pos.clone(),
				message: message.clone(),
			},
			Self::InvalidInstruction {
				pos,
				message,
			} => Self::InvalidInstruction {
				pos: pos.clone(),
				message: message.clone(),
			},
			Self::Symbol {
				pos,
				message,
			} => Self::Symbol {
				pos: pos.clone(),
				message: message.clone(),
			},
			Self::UndefinedSymbol {
				pos,
				symbol,
			} => Self::UndefinedSymbol {
				pos: pos.clone(),
				symbol: symbol.clone(),
			},
			Self::DuplicateSymbol {
				pos,
				symbol,
				previous_pos,
			} => Self::DuplicateSymbol {
				pos: pos.clone(),
				symbol: symbol.clone(),
				previous_pos: previous_pos.clone(),
			},
			Self::Memory {
				pos,
				message,
			} => Self::Memory {
				pos: pos.clone(),
				message: message.clone(),
			},
			Self::BankOverflow {
				pos,
				size,
				available,
			} => Self::BankOverflow {
				pos: pos.clone(),
				size: *size,
				available: *available,
			},
			Self::InvalidAddress {
				pos,
				address,
				message,
			} => Self::InvalidAddress {
				pos: pos.clone(),
				address: *address,
				message: message.clone(),
			},
			Self::Macro {
				pos,
				message,
			} => Self::Macro {
				pos: pos.clone(),
				message: message.clone(),
			},
			Self::Expression {
				pos,
				message,
			} => Self::Expression {
				pos: pos.clone(),
				message: message.clone(),
			},
			Self::NumberParse {
				pos,
				message,
				source: _,
			} => Self::NumberParse {
				pos: pos.clone(),
				message: message.clone(),
				source: None, // Can't clone boxed trait objects
			},
			Self::Output {
				pos,
				message,
			} => Self::Output {
				pos: pos.clone(),
				message: message.clone(),
			},
			Self::Config {
				message,
			} => Self::Config {
				message: message.clone(),
			},
			Self::Platform {
				pos,
				message,
			} => Self::Platform {
				pos: pos.clone(),
				message: message.clone(),
			},
			Self::Internal {
				pos,
				message,
			} => Self::Internal {
				pos: pos.clone(),
				message: message.clone(),
			},
			Self::Multiple {
				errors,
			} => Self::Multiple {
				errors: errors.clone(),
			},
			Self::FirstPassErrors {
				count,
			} => Self::FirstPassErrors {
				count: *count,
			},
			Self::InvalidConfiguration {
				parameter,
			} => Self::InvalidConfiguration {
				parameter: parameter.clone(),
			},
			Self::ParameterError {
				pos,
				message,
			} => Self::ParameterError {
				pos: pos.clone(),
				message: message.clone(),
			},
			Self::ExpansionError {
				pos,
				message,
			} => Self::ExpansionError {
				pos: pos.clone(),
				message: message.clone(),
			},
			Self::InvalidDefinition {
				pos,
				message,
			} => Self::InvalidDefinition {
				pos: pos.clone(),
				message: message.clone(),
			},
			Self::NotFound {
				pos,
				message,
			} => Self::NotFound {
				pos: pos.clone(),
				message: message.clone(),
			},
			Self::InvalidDirectiveParameter {
				pos,
				message,
			} => Self::InvalidDirectiveParameter {
				pos: pos.clone(),
				message: message.clone(),
			},
			Self::InvalidINesParameter {
				pos,
				message,
			} => Self::InvalidINesParameter {
				pos: pos.clone(),
				message: message.clone(),
			},
			Self::InvalidMapper {
				pos,
				message,
			} => Self::InvalidMapper {
				pos: pos.clone(),
				message: message.clone(),
			},
			Self::AlreadyExists {
				name,
			} => Self::AlreadyExists {
				name: name.clone(),
			},
			Self::ScopeError {
				message,
			} => Self::ScopeError {
				message: message.clone(),
			},
			Self::MacroError {
				pos,
				message,
			} => Self::MacroError {
				pos: pos.clone(),
				message: message.clone(),
			},
		}
	}
}

/// Type alias for symbol errors (for compatibility)
pub type SymbolError = AssemblyError;

impl AssemblyError {
	/// Create a new parse error.
	pub fn parse(pos: SourcePos, message: impl Into<String>) -> Self {
		Self::Parse {
			pos,
			message: message.into(),
		}
	}

	/// Create a new symbol error.
	pub fn symbol(pos: SourcePos, message: impl Into<String>) -> Self {
		Self::Symbol {
			pos,
			message: message.into(),
		}
	}

	/// Create a new undefined symbol error.
	pub fn undefined_symbol(pos: SourcePos, symbol: impl Into<String>) -> Self {
		Self::UndefinedSymbol {
			pos,
			symbol: symbol.into(),
		}
	}

	/// Create a new duplicate symbol error.
	pub fn duplicate_symbol(
		pos: SourcePos,
		symbol: impl Into<String>,
		previous_pos: SourcePos,
	) -> Self {
		Self::DuplicateSymbol {
			pos,
			symbol: symbol.into(),
			previous_pos,
		}
	}

	/// Create a new memory error.
	pub fn memory(pos: SourcePos, message: impl Into<String>) -> Self {
		Self::Memory {
			pos,
			message: message.into(),
		}
	}

	/// Create a new bank overflow error.
	pub fn bank_overflow(pos: SourcePos, size: usize, available: usize) -> Self {
		Self::BankOverflow {
			pos,
			size,
			available,
		}
	}

	/// Create a new invalid address error.
	pub fn invalid_address(pos: SourcePos, address: u16, message: impl Into<String>) -> Self {
		Self::InvalidAddress {
			pos,
			address,
			message: message.into(),
		}
	}

	/// Create a new macro error.
	/// Create a new macro error.
	pub fn macro_error(pos: SourcePos, message: impl Into<String>) -> Self {
		Self::MacroError {
			pos,
			message: message.into(),
		}
	}

	/// Create a new expression error.
	pub fn expression(pos: SourcePos, message: impl Into<String>) -> Self {
		Self::Expression {
			pos,
			message: message.into(),
		}
	}

	/// Create a new platform error.
	pub fn platform(pos: SourcePos, message: impl Into<String>) -> Self {
		Self::Platform {
			pos,
			message: message.into(),
		}
	}

	/// Create a new configuration error.
	pub fn config(message: impl Into<String>) -> Self {
		Self::Config {
			message: message.into(),
		}
	}

	/// Create a new internal error.
	pub fn internal(pos: Option<SourcePos>, message: impl Into<String>) -> Self {
		Self::Internal {
			pos,
			message: message.into(),
		}
	}

	/// Get the source position if available.
	pub fn pos(&self) -> Option<&SourcePos> {
		match self {
			Self::Io {
				pos,
				..
			} => pos.as_ref(),
			Self::Parse {
				pos,
				..
			}
			| Self::InvalidInstruction {
				pos,
				..
			}
			| Self::Symbol {
				pos,
				..
			}
			| Self::UndefinedSymbol {
				pos,
				..
			}
			| Self::DuplicateSymbol {
				pos,
				..
			}
			| Self::Memory {
				pos,
				..
			}
			| Self::BankOverflow {
				pos,
				..
			}
			| Self::InvalidAddress {
				pos,
				..
			}
			| Self::Macro {
				pos,
				..
			}
			| Self::Expression {
				pos,
				..
			}
			| Self::NumberParse {
				pos,
				..
			}
			| Self::Platform {
				pos,
				..
			}
			| Self::ParameterError {
				pos,
				..
			}
			| Self::ExpansionError {
				pos,
				..
			}
			| Self::InvalidDefinition {
				pos,
				..
			}
			| Self::NotFound {
				pos,
				..
			}
			| Self::InvalidDirectiveParameter {
				pos,
				..
			}
			| Self::InvalidINesParameter {
				pos,
				..
			}
			| Self::InvalidMapper {
				pos,
				..
			}
			| Self::MacroError {
				pos,
				..
			} => Some(pos),
			Self::Output {
				pos,
				..
			}
			| Self::Internal {
				pos,
				..
			} => pos.as_ref(),
			Self::Config {
				..
			}
			| Self::Multiple {
				..
			}
			| Self::FirstPassErrors {
				..
			}
			| Self::InvalidConfiguration {
				..
			}
			| Self::AlreadyExists {
				..
			}
			| Self::ScopeError {
				..
			} => None,
		}
	}

	/// Check if this is a fatal error that should stop assembly.
	pub fn is_fatal(&self) -> bool {
		matches!(
			self,
			Self::Io { .. } | Self::Config { .. } | Self::Internal { .. } | Self::Multiple { .. }
		)
	}

	/// Combine multiple errors into a single error.
	pub fn multiple(errors: Vec<AssemblyError>) -> Self {
		if errors.len() == 1 {
			errors.into_iter().next().unwrap()
		} else {
			Self::Multiple {
				errors,
			}
		}
	}
}

impl From<std::io::Error> for AssemblyError {
	fn from(error: std::io::Error) -> Self {
		Self::Io {
			pos: None,
			source: error,
		}
	}
}

impl From<ParseIntError> for AssemblyError {
	fn from(error: ParseIntError) -> Self {
		Self::NumberParse {
			pos: SourcePos::file_only(PathBuf::from("<unknown>")),
			message: "Invalid integer".to_string(),
			source: Some(Box::new(error)),
		}
	}
}

impl From<ParseFloatError> for AssemblyError {
	fn from(error: ParseFloatError) -> Self {
		Self::NumberParse {
			pos: SourcePos::file_only(PathBuf::from("<unknown>")),
			message: "Invalid float".to_string(),
			source: Some(Box::new(error)),
		}
	}
}

/// Error collector for gathering multiple errors during assembly.
#[derive(Debug, Default)]
pub struct ErrorCollector {
	errors: Vec<AssemblyError>,
	max_errors: Option<usize>,
}

impl ErrorCollector {
	/// Create a new error collector.
	pub fn new() -> Self {
		Self::default()
	}

	/// Create a new error collector with a maximum error count.
	pub fn with_max_errors(max_errors: usize) -> Self {
		Self {
			errors: Vec::new(),
			max_errors: Some(max_errors),
		}
	}

	/// Add an error to the collector.
	pub fn add(&mut self, error: AssemblyError) -> bool {
		self.errors.push(error);

		if let Some(max) = self.max_errors {
			self.errors.len() >= max
		} else {
			false
		}
	}

	/// Check if there are any errors.
	pub fn has_errors(&self) -> bool {
		!self.errors.is_empty()
	}

	/// Get the number of errors.
	pub fn len(&self) -> usize {
		self.errors.len()
	}

	/// Check if the collector is empty.
	pub fn is_empty(&self) -> bool {
		self.errors.is_empty()
	}

	/// Get all errors.
	pub fn errors(&self) -> &[AssemblyError] {
		&self.errors
	}

	/// Convert to a result, returning an error if any were collected.
	pub fn into_result<T>(self, value: T) -> AssemblyResult<T> {
		if self.errors.is_empty() {
			Ok(value)
		} else {
			Err(AssemblyError::multiple(self.errors))
		}
	}

	/// Convert to a result without consuming self
	pub fn to_result<T>(&self, value: T) -> AssemblyResult<T> {
		if self.errors.is_empty() {
			Ok(value)
		} else {
			Err(AssemblyError::multiple(self.errors.clone()))
		}
	}

	/// Clear all errors.
	pub fn clear(&mut self) {
		self.errors.clear();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::path::Path;

	#[test]
	fn test_source_pos_display() {
		let pos = SourcePos::new(PathBuf::from("test.asm"), 42, 10);
		assert_eq!(format!("{}", pos), "test.asm:42:10");
	}

	#[test]
	fn test_error_creation() {
		let pos = SourcePos::new(PathBuf::from("test.asm"), 1, 1);
		let error = AssemblyError::parse(pos.clone(), "Invalid syntax");

		assert_eq!(error.pos(), Some(&pos));
		assert!(!error.is_fatal());
	}

	#[test]
	fn test_error_collector() {
		let mut collector = ErrorCollector::new();
		assert!(collector.is_empty());

		let pos = SourcePos::new(PathBuf::from("test.asm"), 1, 1);
		collector.add(AssemblyError::parse(pos, "Error 1"));

		assert!(!collector.is_empty());
		assert_eq!(collector.len(), 1);

		let result: AssemblyResult<()> = collector.into_result(());
		assert!(result.is_err());
	}

	#[test]
	fn test_multiple_errors() {
		let pos = SourcePos::new(PathBuf::from("test.asm"), 1, 1);
		let errors = vec![
			AssemblyError::parse(pos.clone(), "Error 1"),
			AssemblyError::parse(pos, "Error 2"),
		];

		let multiple_error = AssemblyError::multiple(errors);
		assert!(matches!(multiple_error, AssemblyError::Multiple { .. }));
		assert!(multiple_error.is_fatal());
	}

	#[test]
	fn test_error_collector_max_errors() {
		let mut collector = ErrorCollector::with_max_errors(2);
		let pos = SourcePos::new(PathBuf::from("test.asm"), 1, 1);

		assert!(!collector.add(AssemblyError::parse(pos.clone(), "Error 1")));
		assert!(collector.add(AssemblyError::parse(pos, "Error 2")));
	}
}
