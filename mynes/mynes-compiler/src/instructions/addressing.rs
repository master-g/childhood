//! Addressing modes for 6502 instructions.
//!
//! This module defines all the addressing modes supported by the 6502 processor,
//! including their encoding patterns and validation rules.

use std::fmt;

use crate::error::{AssemblyError, AssemblyResult, SourcePos};

/// 6502 addressing modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AddressingMode {
	/// Implied - no operand (e.g., NOP, RTS)
	Implied,
	/// Accumulator - operates on accumulator (e.g., ASL A)
	Accumulator,
	/// Immediate - #$nn (e.g., LDA #$20)
	Immediate,
	/// Zero Page - $nn (e.g., LDA $20)
	ZeroPage,
	/// Zero Page,X - $nn,X (e.g., LDA $20,X)
	ZeroPageX,
	/// Zero Page,Y - $nn,Y (e.g., LDX $20,Y)
	ZeroPageY,
	/// Absolute - $nnnn (e.g., LDA $2000)
	Absolute,
	/// Absolute,X - $nnnn,X (e.g., LDA $2000,X)
	AbsoluteX,
	/// Absolute,Y - $nnnn,Y (e.g., LDA $2000,Y)
	AbsoluteY,
	/// Indirect - ($nnnn) (e.g., JMP ($2000))
	Indirect,
	/// Indexed Indirect - ($nn,X) (e.g., LDA ($20,X))
	IndexedIndirect,
	/// Indirect Indexed - ($nn),Y (e.g., LDA ($20),Y)
	IndirectIndexed,
	/// Relative - used for branches (e.g., BNE $20)
	Relative,
}

impl AddressingMode {
	/// Get the size in bytes of the operand for this addressing mode
	pub fn operand_size(&self) -> usize {
		match self {
			Self::Implied | Self::Accumulator => 0,
			Self::Immediate
			| Self::ZeroPage
			| Self::ZeroPageX
			| Self::ZeroPageY
			| Self::IndexedIndirect
			| Self::IndirectIndexed
			| Self::Relative => 1,
			Self::Absolute | Self::AbsoluteX | Self::AbsoluteY | Self::Indirect => 2,
		}
	}

	/// Get the total instruction size including opcode
	pub fn instruction_size(&self) -> usize {
		1 + self.operand_size()
	}

	/// Check if this addressing mode uses a 16-bit address
	pub fn is_16bit_address(&self) -> bool {
		matches!(self, Self::Absolute | Self::AbsoluteX | Self::AbsoluteY | Self::Indirect)
	}

	/// Check if this addressing mode uses zero page
	pub fn is_zero_page(&self) -> bool {
		matches!(
			self,
			Self::ZeroPage
				| Self::ZeroPageX
				| Self::ZeroPageY
				| Self::IndexedIndirect
				| Self::IndirectIndexed
		)
	}

	/// Check if this addressing mode uses indexing
	pub fn is_indexed(&self) -> bool {
		matches!(
			self,
			Self::ZeroPageX
				| Self::ZeroPageY
				| Self::AbsoluteX
				| Self::AbsoluteY
				| Self::IndexedIndirect
				| Self::IndirectIndexed
		)
	}

	/// Check if this addressing mode uses indirection
	pub fn is_indirect(&self) -> bool {
		matches!(self, Self::Indirect | Self::IndexedIndirect | Self::IndirectIndexed)
	}

	/// Check if this addressing mode is for branch instructions
	pub fn is_branch(&self) -> bool {
		matches!(self, Self::Relative)
	}

	/// Get the register used for indexing, if any
	pub fn index_register(&self) -> Option<IndexRegister> {
		match self {
			Self::ZeroPageX | Self::AbsoluteX | Self::IndexedIndirect => Some(IndexRegister::X),
			Self::ZeroPageY | Self::AbsoluteY | Self::IndirectIndexed => Some(IndexRegister::Y),
			_ => None,
		}
	}

	/// Validate that a value is appropriate for this addressing mode
	pub fn validate_operand(&self, value: u16, pos: &SourcePos) -> AssemblyResult<()> {
		match self {
			Self::Implied | Self::Accumulator => {
				// No operand expected
				Ok(())
			}
			Self::Immediate | Self::ZeroPage | Self::ZeroPageX | Self::ZeroPageY => {
				if value > 0xFF {
					Err(AssemblyError::invalid_address(
						pos.clone(),
						value,
						format!("Value ${:04X} too large for {:?} addressing", value, self),
					))
				} else {
					Ok(())
				}
			}
			Self::IndexedIndirect | Self::IndirectIndexed => {
				if value > 0xFF {
					Err(AssemblyError::invalid_address(
						pos.clone(),
						value,
						format!("Zero page address ${:04X} too large for {:?}", value, self),
					))
				} else {
					Ok(())
				}
			}
			Self::Relative => {
				// Relative addressing uses signed 8-bit offset (-128 to +127)
				let signed_value = value as i16;
				if signed_value < -128 || signed_value > 127 {
					Err(AssemblyError::invalid_address(
						pos.clone(),
						value,
						format!("Branch offset {} out of range (-128 to +127)", signed_value),
					))
				} else {
					Ok(())
				}
			}
			Self::Absolute | Self::AbsoluteX | Self::AbsoluteY | Self::Indirect => {
				// 16-bit addresses are always valid for 6502
				Ok(())
			}
		}
	}

	/// Calculate the effective address given base address and operand
	pub fn effective_address(&self, base_addr: u16, operand: u16, x_reg: u8, y_reg: u8) -> u16 {
		match self {
			Self::Implied | Self::Accumulator | Self::Immediate => base_addr,
			Self::ZeroPage => operand,
			Self::ZeroPageX => (operand.wrapping_add(x_reg as u16)) & 0xFF,
			Self::ZeroPageY => (operand.wrapping_add(y_reg as u16)) & 0xFF,
			Self::Absolute => operand,
			Self::AbsoluteX => operand.wrapping_add(x_reg as u16),
			Self::AbsoluteY => operand.wrapping_add(y_reg as u16),
			Self::Indirect => {
				// Note: This would require memory access to resolve
				// For now, just return the operand address
				operand
			}
			Self::IndexedIndirect => {
				// ($nn,X) - add X to zero page address, then read 16-bit address
				(operand.wrapping_add(x_reg as u16)) & 0xFF
			}
			Self::IndirectIndexed => {
				// ($nn),Y - read 16-bit address from zero page, then add Y
				// For calculation purposes, assume we add Y to the base operand
				operand.wrapping_add(y_reg as u16)
			}
			Self::Relative => {
				// For relative addressing, the effective address is base + signed offset
				let offset = operand as i8;
				base_addr.wrapping_add_signed(offset as i16)
			}
		}
	}
}

impl fmt::Display for AddressingMode {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let name = match self {
			Self::Implied => "implied",
			Self::Accumulator => "accumulator",
			Self::Immediate => "immediate",
			Self::ZeroPage => "zero page",
			Self::ZeroPageX => "zero page,X",
			Self::ZeroPageY => "zero page,Y",
			Self::Absolute => "absolute",
			Self::AbsoluteX => "absolute,X",
			Self::AbsoluteY => "absolute,Y",
			Self::Indirect => "indirect",
			Self::IndexedIndirect => "indexed indirect",
			Self::IndirectIndexed => "indirect indexed",
			Self::Relative => "relative",
		};
		write!(f, "{}", name)
	}
}

/// Index registers used in addressing modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexRegister {
	/// X register
	X,
	/// Y register
	Y,
}

impl fmt::Display for IndexRegister {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::X => write!(f, "X"),
			Self::Y => write!(f, "Y"),
		}
	}
}

/// Addressing mode type for categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressingModeType {
	/// No memory access
	Implied,
	/// Immediate value
	Immediate,
	/// Direct memory access
	Direct,
	/// Indexed memory access
	Indexed,
	/// Indirect memory access
	Indirect,
	/// Relative addressing for branches
	Relative,
}

impl AddressingMode {
	/// Get the addressing mode type category
	pub fn mode_type(&self) -> AddressingModeType {
		match self {
			Self::Implied | Self::Accumulator => AddressingModeType::Implied,
			Self::Immediate => AddressingModeType::Immediate,
			Self::ZeroPage | Self::Absolute => AddressingModeType::Direct,
			Self::ZeroPageX | Self::ZeroPageY | Self::AbsoluteX | Self::AbsoluteY => {
				AddressingModeType::Indexed
			}
			Self::Indirect | Self::IndexedIndirect | Self::IndirectIndexed => {
				AddressingModeType::Indirect
			}
			Self::Relative => AddressingModeType::Relative,
		}
	}
}

/// Addressing mode parser for assembly syntax
pub struct AddressingModeParser;

impl AddressingModeParser {
	/// Parse addressing mode from operand string
	pub fn parse(operand: &str) -> Option<(AddressingMode, String)> {
		let trimmed = operand.trim();

		if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("A") {
			// Implied or Accumulator
			if trimmed.eq_ignore_ascii_case("A") {
				Some((AddressingMode::Accumulator, String::new()))
			} else {
				Some((AddressingMode::Implied, String::new()))
			}
		} else if trimmed.starts_with('#') {
			// Immediate - #$nn or #value
			Some((AddressingMode::Immediate, trimmed[1..].to_string()))
		} else if trimmed.starts_with('(') && trimmed.ends_with(')') {
			// Some form of indirect addressing
			let inner = &trimmed[1..trimmed.len() - 1];
			if inner.ends_with(",X") {
				// ($nn,X) - Indexed Indirect
				let addr = inner[..inner.len() - 2].trim();
				Some((AddressingMode::IndexedIndirect, addr.to_string()))
			} else {
				// ($nnnn) - Indirect
				Some((AddressingMode::Indirect, inner.to_string()))
			}
		} else if trimmed.starts_with('(') && trimmed.ends_with("),Y") {
			// ($nn),Y - Indirect Indexed
			let addr_part = &trimmed[1..trimmed.len() - 3];
			Some((AddressingMode::IndirectIndexed, addr_part.to_string()))
		} else if trimmed.ends_with(",X") {
			// $nn,X or $nnnn,X
			let addr = trimmed[..trimmed.len() - 2].trim();
			if Self::is_zero_page_value(addr) {
				Some((AddressingMode::ZeroPageX, addr.to_string()))
			} else {
				Some((AddressingMode::AbsoluteX, addr.to_string()))
			}
		} else if trimmed.ends_with(",Y") {
			// $nn,Y or $nnnn,Y
			let addr = trimmed[..trimmed.len() - 2].trim();
			if Self::is_zero_page_value(addr) {
				Some((AddressingMode::ZeroPageY, addr.to_string()))
			} else {
				Some((AddressingMode::AbsoluteY, addr.to_string()))
			}
		} else {
			// Direct addressing - determine if zero page or absolute
			if Self::is_zero_page_value(trimmed) {
				Some((AddressingMode::ZeroPage, trimmed.to_string()))
			} else {
				Some((AddressingMode::Absolute, trimmed.to_string()))
			}
		}
	}

	/// Check if a value string represents a zero page address
	fn is_zero_page_value(value: &str) -> bool {
		// Try to parse as hex or decimal and check if <= 0xFF
		if let Some(hex_str) = value.strip_prefix('$') {
			if let Ok(val) = u16::from_str_radix(hex_str, 16) {
				return val <= 0xFF;
			}
		}

		if let Ok(val) = value.parse::<u16>() {
			return val <= 0xFF;
		}

		// If we can't parse it, assume it might be a symbol
		// In a real assembler, this would be resolved later
		false
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
	fn test_addressing_mode_sizes() {
		assert_eq!(AddressingMode::Implied.operand_size(), 0);
		assert_eq!(AddressingMode::Accumulator.operand_size(), 0);
		assert_eq!(AddressingMode::Immediate.operand_size(), 1);
		assert_eq!(AddressingMode::ZeroPage.operand_size(), 1);
		assert_eq!(AddressingMode::ZeroPageX.operand_size(), 1);
		assert_eq!(AddressingMode::Absolute.operand_size(), 2);
		assert_eq!(AddressingMode::AbsoluteX.operand_size(), 2);
		assert_eq!(AddressingMode::Indirect.operand_size(), 2);
		assert_eq!(AddressingMode::IndexedIndirect.operand_size(), 1);
		assert_eq!(AddressingMode::IndirectIndexed.operand_size(), 1);
		assert_eq!(AddressingMode::Relative.operand_size(), 1);
	}

	#[test]
	fn test_addressing_mode_properties() {
		assert!(AddressingMode::Absolute.is_16bit_address());
		assert!(!AddressingMode::ZeroPage.is_16bit_address());

		assert!(AddressingMode::ZeroPage.is_zero_page());
		assert!(!AddressingMode::Absolute.is_zero_page());

		assert!(AddressingMode::AbsoluteX.is_indexed());
		assert!(!AddressingMode::Absolute.is_indexed());

		assert!(AddressingMode::Indirect.is_indirect());
		assert!(!AddressingMode::Absolute.is_indirect());

		assert!(AddressingMode::Relative.is_branch());
		assert!(!AddressingMode::Absolute.is_branch());
	}

	#[test]
	fn test_index_registers() {
		assert_eq!(AddressingMode::ZeroPageX.index_register(), Some(IndexRegister::X));
		assert_eq!(AddressingMode::AbsoluteY.index_register(), Some(IndexRegister::Y));
		assert_eq!(AddressingMode::IndexedIndirect.index_register(), Some(IndexRegister::X));
		assert_eq!(AddressingMode::IndirectIndexed.index_register(), Some(IndexRegister::Y));
		assert_eq!(AddressingMode::Absolute.index_register(), None);
	}

	#[test]
	fn test_operand_validation() {
		let pos = test_pos();

		// Valid zero page address
		assert!(AddressingMode::ZeroPage.validate_operand(0x80, &pos).is_ok());

		// Invalid zero page address (too large)
		assert!(AddressingMode::ZeroPage.validate_operand(0x200, &pos).is_err());

		// Valid absolute address
		assert!(AddressingMode::Absolute.validate_operand(0x8000, &pos).is_ok());

		// Valid relative branch
		assert!(AddressingMode::Relative.validate_operand(10, &pos).is_ok());

		// Invalid relative branch (too large)
		assert!(AddressingMode::Relative.validate_operand(200, &pos).is_err());
	}

	#[test]
	fn test_effective_address() {
		// Zero page
		assert_eq!(AddressingMode::ZeroPage.effective_address(0x8000, 0x80, 0, 0), 0x80);

		// Zero page,X
		assert_eq!(AddressingMode::ZeroPageX.effective_address(0x8000, 0x80, 0x10, 0), 0x90);

		// Absolute
		assert_eq!(AddressingMode::Absolute.effective_address(0x8000, 0x2000, 0, 0), 0x2000);

		// Absolute,X
		assert_eq!(AddressingMode::AbsoluteX.effective_address(0x8000, 0x2000, 0x10, 0), 0x2010);

		// Relative
		assert_eq!(AddressingMode::Relative.effective_address(0x8000, 10, 0, 0), 0x800A);
		assert_eq!(
			AddressingMode::Relative.effective_address(0x8000, (-10_i8) as u8 as u16, 0, 0),
			0x7FF6
		);
	}

	#[test]
	fn test_addressing_mode_types() {
		assert_eq!(AddressingMode::Implied.mode_type(), AddressingModeType::Implied);
		assert_eq!(AddressingMode::Immediate.mode_type(), AddressingModeType::Immediate);
		assert_eq!(AddressingMode::Absolute.mode_type(), AddressingModeType::Direct);
		assert_eq!(AddressingMode::AbsoluteX.mode_type(), AddressingModeType::Indexed);
		assert_eq!(AddressingMode::Indirect.mode_type(), AddressingModeType::Indirect);
		assert_eq!(AddressingMode::Relative.mode_type(), AddressingModeType::Relative);
	}

	#[test]
	fn test_addressing_mode_parser() {
		// Implied
		assert_eq!(AddressingModeParser::parse(""), Some((AddressingMode::Implied, String::new())));

		// Accumulator
		assert_eq!(
			AddressingModeParser::parse("A"),
			Some((AddressingMode::Accumulator, String::new()))
		);

		// Immediate
		assert_eq!(
			AddressingModeParser::parse("#$20"),
			Some((AddressingMode::Immediate, "$20".to_string()))
		);

		// Zero page
		assert_eq!(
			AddressingModeParser::parse("$80"),
			Some((AddressingMode::ZeroPage, "$80".to_string()))
		);

		// Absolute
		assert_eq!(
			AddressingModeParser::parse("$2000"),
			Some((AddressingMode::Absolute, "$2000".to_string()))
		);

		// Zero page,X
		assert_eq!(
			AddressingModeParser::parse("$80,X"),
			Some((AddressingMode::ZeroPageX, "$80".to_string()))
		);

		// Absolute,X
		assert_eq!(
			AddressingModeParser::parse("$2000,X"),
			Some((AddressingMode::AbsoluteX, "$2000".to_string()))
		);

		// Absolute,Y
		assert_eq!(
			AddressingModeParser::parse("$2000,Y"),
			Some((AddressingMode::AbsoluteY, "$2000".to_string()))
		);

		// Indirect
		assert_eq!(
			AddressingModeParser::parse("($2000)"),
			Some((AddressingMode::Indirect, "$2000".to_string()))
		);

		// Indexed Indirect
		assert_eq!(
			AddressingModeParser::parse("($80,X)"),
			Some((AddressingMode::IndexedIndirect, "$80".to_string()))
		);

		// Indirect Indexed
		assert_eq!(
			AddressingModeParser::parse("($80),Y"),
			Some((AddressingMode::IndirectIndexed, "$80".to_string()))
		);
	}

	#[test]
	fn test_display() {
		assert_eq!(format!("{}", AddressingMode::Implied), "implied");
		assert_eq!(format!("{}", AddressingMode::Immediate), "immediate");
		assert_eq!(format!("{}", AddressingMode::ZeroPageX), "zero page,X");
		assert_eq!(format!("{}", AddressingMode::AbsoluteY), "absolute,Y");
		assert_eq!(format!("{}", AddressingMode::IndexedIndirect), "indexed indirect");
		assert_eq!(format!("{}", AddressingMode::IndirectIndexed), "indirect indexed");
	}
}
