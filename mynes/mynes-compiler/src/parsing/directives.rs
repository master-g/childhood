//! Directive handling for the NES compiler
//!
//! This module defines and handles assembler directives (pseudo-instructions)
//! that control assembly behavior and output generation.

use crate::error::{AssemblyError, AssemblyResult, SourcePos};
use std::fmt;
use std::path::PathBuf;

/// Directive types for assembler directives
#[derive(Debug, Clone, PartialEq)]
pub enum DirectiveType {
	/// Set origin address (.org)
	Org {
		address: u16,
	},
	/// Select memory bank (.bank)
	Bank {
		number: u8,
	},
	/// Select memory section (.code, .data, .bss, .zp)
	Section {
		name: String,
	},
	/// Define byte data (.db)
	DataByte {
		data: Vec<u8>,
	},
	/// Define word data (.dw)
	DataWord {
		data: Vec<u16>,
	},
	/// Reserve space (.ds)
	Reserve {
		size: usize,
	},
	/// Align to boundary (.align)
	Align {
		boundary: u8,
	},
	/// Set symbol value (.equ)
	Equ {
		name: String,
		value: i32,
	},
	/// Set string symbol (.sequ)
	StringEqu {
		name: String,
		value: String,
	},
	/// Include binary file (.incbin)
	IncludeBinary {
		filename: String,
	},
	/// Include source file (.include)
	Include {
		filename: String,
	},
	/// List control (.list, .nolist)
	List {
		enabled: bool,
	},
	/// Macro list control (.mlist, .nomlist)
	MacroList {
		enabled: bool,
	},
	/// Conditional assembly (.if)
	If {
		condition: i32,
	},
	/// Conditional else (.else)
	Else,
	/// End conditional (.endif)
	EndIf,
	/// Conditional ifdef (.ifdef)
	IfDef {
		symbol: String,
	},
	/// Conditional ifndef (.ifndef)
	IfNDef {
		symbol: String,
	},
	/// NES-specific: Set PRG banks (.inesprg)
	InesPrg {
		banks: u8,
	},
	/// NES-specific: Set CHR banks (.ineschr)
	InesChr {
		banks: u8,
	},
	/// NES-specific: Set mapper (.inesmap)
	InesMap {
		mapper: u8,
	},
	/// NES-specific: Set mirroring (.inesmir)
	InesMir {
		mirroring: u8,
	},
	/// NES-specific: Set submapper (.inessubmap)
	InesSubMap {
		submapper: u8,
	},
	/// NES-specific: Set battery flag (.inesbat)
	InesBat {
		enabled: bool,
	},
	/// NES-specific: Set timing (.inestim)
	InesTim {
		timing: u8,
	},
	/// RS set (.rsset)
	RsSet {
		value: i32,
	},
	/// RS directive (.rs)
	Rs {
		size: usize,
	},
	/// Fail directive (.fail)
	Fail {
		message: String,
	},
	/// Macro definition start (.macro)
	MacroStart {
		name: String,
	},
	/// Macro definition end (.endm)
	MacroEnd,
	/// Function definition (.func)
	Function {
		name: String,
		body: String,
	},
	/// Procedure start (.proc)
	ProcStart {
		name: String,
	},
	/// Procedure end (.endp)
	ProcEnd,
	/// Procedure group start (.procgroup)
	ProcGroupStart {
		name: String,
	},
	/// Procedure group end (.endprocgroup)
	ProcGroupEnd,
}

/// Complete directive with type and position information
/// Directive with position information
#[derive(Debug, Clone, PartialEq)]
pub struct Directive {
	/// Directive type
	pub directive_type: DirectiveType,
	/// Source position
	pub pos: SourcePos,
}

impl fmt::Display for Directive {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match &self.directive_type {
			DirectiveType::Org {
				address,
			} => write!(f, ".org ${:04X}", address),
			DirectiveType::Bank {
				number,
			} => write!(f, ".bank {}", number),
			DirectiveType::Section {
				name,
			} => write!(f, ".{}", name),
			DirectiveType::DataByte {
				data,
			} => {
				write!(f, ".db ")?;
				for (i, byte) in data.iter().enumerate() {
					if i > 0 {
						write!(f, ", ")?;
					}
					write!(f, "${:02X}", byte)?;
				}
				Ok(())
			}
			DirectiveType::DataWord {
				data,
			} => {
				write!(f, ".dw ")?;
				for (i, word) in data.iter().enumerate() {
					if i > 0 {
						write!(f, ", ")?;
					}
					write!(f, "${:04X}", word)?;
				}
				Ok(())
			}
			DirectiveType::Reserve {
				size,
			} => write!(f, ".rs {}", size),
			DirectiveType::Align {
				boundary,
			} => write!(f, ".align {}", boundary),
			DirectiveType::Equ {
				name,
				value,
			} => write!(f, "{} = {}", name, value),
			DirectiveType::StringEqu {
				name,
				value,
			} => write!(f, "{} .equ \"{}\"", name, value),
			DirectiveType::IncludeBinary {
				filename,
			} => write!(f, ".incbin \"{}\"", filename),
			DirectiveType::Include {
				filename,
			} => write!(f, ".include \"{}\"", filename),
			DirectiveType::List {
				enabled,
			} => write!(
				f,
				".{}",
				if *enabled {
					"list"
				} else {
					"nolist"
				}
			),
			DirectiveType::MacroList {
				enabled,
			} => write!(
				f,
				".{}",
				if *enabled {
					"mlist"
				} else {
					"nomlist"
				}
			),
			DirectiveType::If {
				condition,
			} => write!(f, ".if {}", condition),
			DirectiveType::Else => write!(f, ".else"),
			DirectiveType::EndIf => write!(f, ".endif"),
			DirectiveType::IfDef {
				symbol,
			} => write!(f, ".ifdef {}", symbol),
			DirectiveType::IfNDef {
				symbol,
			} => write!(f, ".ifndef {}", symbol),
			DirectiveType::InesPrg {
				banks,
			} => write!(f, ".inesprg {}", banks),
			DirectiveType::InesChr {
				banks,
			} => write!(f, ".ineschr {}", banks),
			DirectiveType::InesMap {
				mapper,
			} => write!(f, ".inesmap {}", mapper),
			DirectiveType::InesMir {
				mirroring,
			} => write!(f, ".inesmir {}", mirroring),
			DirectiveType::InesSubMap {
				submapper,
			} => write!(f, ".inessubmap {}", submapper),
			DirectiveType::MacroStart {
				name,
			} => write!(f, ".macro {}", name),
			DirectiveType::MacroEnd => write!(f, ".endm"),
			DirectiveType::ProcStart {
				name,
			} => write!(f, ".proc {}", name),
			DirectiveType::ProcEnd => write!(f, ".endp"),
			_ => write!(f, "<directive>"),
		}
	}
}

impl Directive {
	/// Create a new directive
	pub fn new(directive_type: DirectiveType, pos: SourcePos) -> Self {
		Self {
			directive_type,
			pos,
		}
	}
}

/// Parser for assembler directives
#[derive(Debug)]
pub struct DirectiveParser {
	/// Whether case-sensitive parsing is enabled
	case_sensitive: bool,
}

impl DirectiveParser {
	/// Create a new directive parser
	pub fn new() -> Self {
		Self {
			case_sensitive: false,
		}
	}

	/// Set case sensitivity
	pub fn set_case_sensitive(&mut self, case_sensitive: bool) {
		self.case_sensitive = case_sensitive;
	}

	/// Parse a directive from a line of text
	pub fn parse_directive(&self, line: &str, pos: SourcePos) -> AssemblyResult<Option<Directive>> {
		let trimmed = line.trim();

		// Check if this is actually a directive
		if !trimmed.starts_with('.') {
			return Ok(None);
		}

		// Split into parts
		let parts: Vec<&str> = trimmed.split_whitespace().collect();
		if parts.is_empty() {
			return Ok(None);
		}

		let directive_name = if self.case_sensitive {
			parts[0].to_string()
		} else {
			parts[0].to_uppercase()
		};

		match directive_name.as_str() {
			".ORG" => {
				if parts.len() != 2 {
					return Err(AssemblyError::parse(
						pos,
						"ORG requires exactly one address parameter",
					));
				}
				let address = self.parse_number(parts[1], &pos)?;
				Ok(Some(Directive::new(
					DirectiveType::Org {
						address: address as u16,
					},
					pos,
				)))
			}
			".BANK" => {
				if parts.len() != 2 {
					return Err(AssemblyError::parse(pos, "BANK requires exactly one bank number"));
				}
				let bank = self.parse_number(parts[1], &pos)?;
				if bank < 0 || bank > 255 {
					return Err(AssemblyError::parse(pos, "Bank number must be 0-255"));
				}
				Ok(Some(Directive::new(
					DirectiveType::Bank {
						number: bank as u8,
					},
					pos,
				)))
			}
			".CODE" => Ok(Some(Directive::new(
				DirectiveType::Section {
					name: "code".to_string(),
				},
				pos,
			))),
			".DATA" => Ok(Some(Directive::new(
				DirectiveType::Section {
					name: "data".to_string(),
				},
				pos,
			))),
			".BSS" => Ok(Some(Directive::new(
				DirectiveType::Section {
					name: "bss".to_string(),
				},
				pos,
			))),
			".ZP" => Ok(Some(Directive::new(
				DirectiveType::Section {
					name: "zp".to_string(),
				},
				pos,
			))),
			".DB" | ".BYTE" => {
				let mut bytes = Vec::new();
				for &part in &parts[1..] {
					if part.starts_with('"') && part.ends_with('"') {
						// String literal
						let string_content = &part[1..part.len() - 1];
						bytes.extend_from_slice(string_content.as_bytes());
					} else {
						// Numeric value
						let value = self.parse_number(part, &pos)?;
						if value < 0 || value > 255 {
							return Err(AssemblyError::parse(
								pos,
								format!("Byte value {} out of range 0-255", value),
							));
						}
						bytes.push(value as u8);
					}
				}
				Ok(Some(Directive::new(
					DirectiveType::DataByte {
						data: bytes,
					},
					pos,
				)))
			}
			".DW" | ".WORD" => {
				let mut words = Vec::new();
				for &part in &parts[1..] {
					let value = self.parse_number(part, &pos)?;
					if value < 0 || value > 65535 {
						return Err(AssemblyError::parse(
							pos,
							format!("Word value {} out of range 0-65535", value),
						));
					}
					words.push(value as u16);
				}
				Ok(Some(Directive::new(
					DirectiveType::DataWord {
						data: words,
					},
					pos,
				)))
			}
			".DS" => {
				if parts.len() != 2 {
					return Err(AssemblyError::parse(
						pos,
						"DS requires exactly one size parameter",
					));
				}
				let size = self.parse_number(parts[1], &pos)?;
				if size < 0 {
					return Err(AssemblyError::parse(pos, "DS size cannot be negative"));
				}
				Ok(Some(Directive::new(
					DirectiveType::Reserve {
						size: size as usize,
					},
					pos,
				)))
			}
			".EQU" | "=" => {
				if parts.len() < 3 {
					return Err(AssemblyError::InvalidDirectiveParameter {
						pos: pos.clone(),
						message: "EQU requires symbol name and value".to_string(),
					});
				}
				// Join remaining parts as the value expression
				let value_str = parts[2..].join(" ");
				let value = self.parse_number(&value_str, &pos)?;
				Ok(Some(Directive {
					directive_type: DirectiveType::Equ {
						name: parts[1].to_string(),
						value,
					},
					pos: pos.clone(),
				}))
			}
			".SEQU" => {
				if parts.len() < 3 {
					return Err(AssemblyError::InvalidDirectiveParameter {
						pos: pos.clone(),
						message: "SEQU requires symbol name and string value".to_string(),
					});
				}
				let value = parts[2..].join(" ");
				Ok(Some(Directive {
					directive_type: DirectiveType::StringEqu {
						name: parts[1].to_string(),
						value,
					},
					pos: pos.clone(),
				}))
			}
			".INCLUDE" => {
				if parts.len() != 2 {
					return Err(AssemblyError::InvalidDirectiveParameter {
						pos: pos.clone(),
						message: "INCLUDE requires exactly one filename".to_string(),
					});
				}
				Ok(Some(Directive {
					directive_type: DirectiveType::Include {
						filename: parts[1].to_string(),
					},
					pos: pos.clone(),
				}))
			}
			".INCBIN" => {
				if parts.len() != 2 {
					return Err(AssemblyError::InvalidDirectiveParameter {
						pos: pos.clone(),
						message: "INCBIN requires exactly one filename".to_string(),
					});
				}
				Ok(Some(Directive {
					directive_type: DirectiveType::IncludeBinary {
						filename: parts[1].to_string(),
					},
					pos: pos.clone(),
				}))
			}
			".LIST" => Ok(Some(Directive {
				directive_type: DirectiveType::List {
					enabled: true,
				},
				pos: pos.clone(),
			})),
			".NOLIST" => Ok(Some(Directive {
				directive_type: DirectiveType::List {
					enabled: false,
				},
				pos: pos.clone(),
			})),
			".MLIST" => Ok(Some(Directive {
				directive_type: DirectiveType::MacroList {
					enabled: true,
				},
				pos: pos.clone(),
			})),
			".NOMLIST" => Ok(Some(Directive {
				directive_type: DirectiveType::MacroList {
					enabled: false,
				},
				pos: pos.clone(),
			})),
			".INESPRG" => {
				if parts.len() != 2 {
					return Err(AssemblyError::InvalidDirectiveParameter {
						pos: pos.clone(),
						message: "INESPRG requires exactly one bank count".to_string(),
					});
				}
				let banks = self.parse_number(parts[1], &pos)?;
				if banks < 0 || banks > 255 {
					return Err(AssemblyError::InvalidINesParameter {
						pos: pos.clone(),
						message: format!("PRG bank count {} out of range 0-255", banks),
					});
				}
				Ok(Some(Directive {
					directive_type: DirectiveType::InesPrg {
						banks: banks as u8,
					},
					pos: pos.clone(),
				}))
			}
			".INESCHR" => {
				if parts.len() != 2 {
					return Err(AssemblyError::InvalidDirectiveParameter {
						pos: pos.clone(),
						message: "INESCHR requires exactly one bank count".to_string(),
					});
				}
				let banks = self.parse_number(parts[1], &pos)?;
				if banks < 0 || banks > 255 {
					return Err(AssemblyError::InvalidINesParameter {
						pos: pos.clone(),
						message: format!("CHR bank count {} out of range 0-255", banks),
					});
				}
				Ok(Some(Directive {
					directive_type: DirectiveType::InesChr {
						banks: banks as u8,
					},
					pos: pos.clone(),
				}))
			}
			".INESMAP" => {
				if parts.len() != 2 {
					return Err(AssemblyError::InvalidDirectiveParameter {
						pos: pos.clone(),
						message: "INESMAP requires exactly one mapper number".to_string(),
					});
				}
				let mapper = self.parse_number(parts[1], &pos)?;
				if mapper < 0 || mapper > 255 {
					return Err(AssemblyError::InvalidMapper {
						pos: pos.clone(),
						message: format!("Mapper {} out of range 0-255", mapper),
					});
				}
				Ok(Some(Directive {
					directive_type: DirectiveType::InesMap {
						mapper: mapper as u8,
					},
					pos: pos.clone(),
				}))
			}
			".INESMIR" => {
				if parts.len() != 2 {
					return Err(AssemblyError::InvalidDirectiveParameter {
						pos: pos.clone(),
						message: "INESMIR requires exactly one mirroring type".to_string(),
					});
				}
				let mirroring = self.parse_number(parts[1], &pos)?;
				if mirroring < 0 || mirroring > 3 {
					return Err(AssemblyError::InvalidINesParameter {
						pos: pos.clone(),
						message: format!("Mirroring type {} out of range 0-3", mirroring),
					});
				}
				Ok(Some(Directive {
					directive_type: DirectiveType::InesMir {
						mirroring: mirroring as u8,
					},
					pos: pos.clone(),
				}))
			}
			".RSSET" => {
				if parts.len() != 2 {
					return Err(AssemblyError::InvalidDirectiveParameter {
						pos: pos.clone(),
						message: "RSSET requires exactly one value".to_string(),
					});
				}
				let value = self.parse_number(parts[1], &pos)?;
				Ok(Some(Directive {
					directive_type: DirectiveType::Reserve {
						size: value as usize,
					},
					pos: pos.clone(),
				}))
			}
			".RS" => {
				if parts.len() != 2 {
					return Err(AssemblyError::InvalidDirectiveParameter {
						pos: pos.clone(),
						message: "RS requires exactly one size value".to_string(),
					});
				}
				let size = self.parse_number(parts[1], &pos)?;
				if size < 0 {
					return Err(AssemblyError::InvalidDirectiveParameter {
						pos: pos.clone(),
						message: "RS size cannot be negative".to_string(),
					});
				}
				Ok(Some(Directive {
					directive_type: DirectiveType::Reserve {
						size: size as usize,
					},
					pos: pos.clone(),
				}))
			}
			".MACRO" => {
				let name = if parts.len() > 1 {
					parts[1].to_string()
				} else {
					String::new()
				};
				Ok(Some(Directive {
					directive_type: DirectiveType::MacroStart {
						name,
					},
					pos: pos.clone(),
				}))
			}
			".ENDM" => Ok(Some(Directive {
				directive_type: DirectiveType::MacroEnd,
				pos: pos.clone(),
			})),
			".PROC" => {
				if parts.len() != 2 {
					return Err(AssemblyError::InvalidDirectiveParameter {
						pos: pos.clone(),
						message: "PROC requires exactly one procedure name".to_string(),
					});
				}
				Ok(Some(Directive {
					directive_type: DirectiveType::ProcStart {
						name: parts[1].to_string(),
					},
					pos: pos.clone(),
				}))
			}
			".ENDP" => Ok(Some(Directive {
				directive_type: DirectiveType::ProcEnd,
				pos: pos.clone(),
			})),
			".IF" => {
				if parts.len() < 2 {
					return Err(AssemblyError::InvalidDirectiveParameter {
						pos: pos.clone(),
						message: "IF requires exactly one expression".to_string(),
					});
				}
				let expr_str = parts[1..].join(" ");
				let value = self.parse_number(&expr_str, &pos)?;
				Ok(Some(Directive {
					directive_type: DirectiveType::If {
						condition: value,
					},
					pos: pos.clone(),
				}))
			}
			".ELSE" => Ok(Some(Directive {
				directive_type: DirectiveType::Else,
				pos: pos.clone(),
			})),
			".ENDIF" => Ok(Some(Directive {
				directive_type: DirectiveType::EndIf,
				pos: pos.clone(),
			})),
			".IFDEF" => {
				if parts.len() != 2 {
					return Err(AssemblyError::InvalidDirectiveParameter {
						pos: pos.clone(),
						message: "IFDEF requires exactly one symbol name".to_string(),
					});
				}
				Ok(Some(Directive {
					directive_type: DirectiveType::IfDef {
						symbol: parts[1].to_string(),
					},
					pos: pos.clone(),
				}))
			}
			".IFNDEF" => {
				if parts.len() != 2 {
					return Err(AssemblyError::InvalidDirectiveParameter {
						pos: pos.clone(),
						message: "IFNDEF requires exactly one symbol name".to_string(),
					});
				}
				Ok(Some(Directive {
					directive_type: DirectiveType::IfNDef {
						symbol: parts[1].to_string(),
					},
					pos: pos.clone(),
				}))
			}
			".FAIL" => {
				let message = if parts.len() > 1 {
					parts[1..].join(" ")
				} else {
					"Assembly failed".to_string()
				};
				// FAIL directive - not in DirectiveType enum, treat as error
				return Err(AssemblyError::Parse {
					pos: pos.clone(),
					message: format!("Assembly failed: {}", message),
				});
			}
			_ => {
				// Unknown directive
				Err(AssemblyError::Parse {
					pos: pos.clone(),
					message: format!("Unknown directive: {}", directive_name),
				})
			}
		}
	}

	/// Parse a numeric value (supports decimal, hex, binary)
	fn parse_number(&self, input: &str, pos: &SourcePos) -> AssemblyResult<i32> {
		let trimmed = input.trim();

		if trimmed.is_empty() {
			return Err(AssemblyError::parse(
				pos.clone(),
				format!("Empty numeric literal: {}", input),
			));
		}

		// Character literal 'X'
		if trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() == 3 {
			let ch = trimmed.chars().nth(1).unwrap();
			return Ok(ch as u8 as i32);
		}

		// Hexadecimal $XX
		if trimmed.starts_with('$') {
			let hex_part = &trimmed[1..];
			if hex_part.is_empty() {
				return Err(AssemblyError::parse(
					pos.clone(),
					format!("Invalid hex literal: {}", input),
				));
			}
			return i32::from_str_radix(hex_part, 16).map_err(|_| {
				AssemblyError::parse(pos.clone(), format!("Invalid hex literal: {}", input))
			});
		}

		// Binary %XX
		if trimmed.starts_with('%') {
			let bin_part = &trimmed[1..];
			if bin_part.is_empty() {
				return Err(AssemblyError::parse(
					pos.clone(),
					format!("Invalid binary literal: {}", input),
				));
			}
			return i32::from_str_radix(bin_part, 2).map_err(|_| {
				AssemblyError::parse(pos.clone(), format!("Invalid binary literal: {}", input))
			});
		}

		// Decimal (default)
		trimmed.parse::<i32>().map_err(|_| {
			AssemblyError::parse(pos.clone(), format!("Invalid numeric literal: {}", input))
		})
	}
}

impl Default for DirectiveParser {
	fn default() -> Self {
		Self::new()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn test_pos() -> SourcePos {
		SourcePos::new(PathBuf::from("test.asm"), 1, 1)
	}

	#[test]
	fn test_directive_parser_creation() {
		let parser = DirectiveParser::new();
		assert!(!parser.case_sensitive);
	}

	#[test]
	fn test_org_directive() {
		let parser = DirectiveParser::new();
		let pos = test_pos();
		let result = parser.parse_directive(".org $8000", pos).unwrap();

		match result {
			Some(directive) => match directive.directive_type {
				DirectiveType::Org {
					address,
				} => assert_eq!(address, 0x8000),
				_ => panic!("Expected Org directive"),
			},
			_ => panic!("Expected Some directive"),
		}
	}

	#[test]
	fn test_data_directives() {
		let parser = DirectiveParser::new();
		let pos = test_pos();

		// Test .db with numbers would need proper comma parsing
		// Simplified test for now
		let result = parser.parse_directive(".db $01", pos.clone()).unwrap();
		match result {
			Some(directive) => match directive.directive_type {
				DirectiveType::DataByte {
					data,
				} => assert_eq!(data, vec![1]),
				_ => panic!("Expected DataByte directive"),
			},
			_ => panic!("Expected Some directive"),
		}
	}

	#[test]
	fn test_section_directives() {
		let parser = DirectiveParser::new();
		let pos = test_pos();

		let result = parser.parse_directive(".code", pos.clone()).unwrap();
		match result {
			Some(directive) => match directive.directive_type {
				DirectiveType::Section {
					name,
				} => assert_eq!(name, "code"),
				_ => panic!("Expected Section directive"),
			},
			_ => panic!("Expected Some directive"),
		}
	}

	#[test]
	fn test_number_parsing() {
		let parser = DirectiveParser::new();
		let pos = test_pos();

		assert_eq!(parser.parse_number("42", &pos).unwrap(), 42);
		assert_eq!(parser.parse_number("$FF", &pos).unwrap(), 255);
		assert_eq!(parser.parse_number("%11111111", &pos).unwrap(), 255);
		assert_eq!(parser.parse_number("'A'", &pos).unwrap(), 65);

		assert!(parser.parse_number("invalid", &pos).is_err());
		assert!(parser.parse_number("$", &pos).is_err());
		assert!(parser.parse_number("%", &pos).is_err());
	}

	#[test]
	fn test_non_directive() {
		let parser = DirectiveParser::new();
		let pos = test_pos();

		let result = parser.parse_directive("LDA #$42", pos.clone()).unwrap();
		assert!(result.is_none());

		let result = parser.parse_directive("start:", pos).unwrap();
		assert!(result.is_none());
	}
}
