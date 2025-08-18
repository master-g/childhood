//! Instruction encoding for 6502 instructions.
//!
//! This module handles the encoding of 6502 instructions into machine code,
//! including operand encoding and address resolution.

use crate::error::{AssemblyError, AssemblyResult, SourcePos};
use crate::instructions::Operand;
use crate::instructions::addressing::AddressingMode;
use crate::instructions::opcodes::{Mnemonic, get_opcode};

/// Instruction encoder for generating machine code
pub struct InstructionEncoder;

impl InstructionEncoder {
	/// Encode a single instruction to machine code bytes
	pub fn encode(
		mnemonic: Mnemonic,
		addressing_mode: AddressingMode,
		operand: &Operand,
		pos: &SourcePos,
	) -> AssemblyResult<Vec<u8>> {
		// Get the opcode information
		let opcode_info = get_opcode(mnemonic, addressing_mode).ok_or_else(|| {
			AssemblyError::InvalidInstruction {
				pos: pos.clone(),
				message: format!(
					"Invalid addressing mode {} for instruction {}",
					addressing_mode, mnemonic
				),
			}
		})?;

		let mut bytes = vec![opcode_info.opcode];

		// Encode the operand based on addressing mode
		match addressing_mode {
			AddressingMode::Implied | AddressingMode::Accumulator => {
				// No operand bytes
			}
			AddressingMode::Immediate => {
				let value = Self::extract_byte_operand(operand, pos)?;
				bytes.push(value);
			}
			AddressingMode::ZeroPage
			| AddressingMode::ZeroPageX
			| AddressingMode::ZeroPageY
			| AddressingMode::IndexedIndirect
			| AddressingMode::IndirectIndexed => {
				let value = Self::extract_byte_operand(operand, pos)?;
				bytes.push(value);
			}
			AddressingMode::Relative => {
				let offset = Self::extract_relative_operand(operand, pos)?;
				bytes.push(offset as u8);
			}
			AddressingMode::Absolute
			| AddressingMode::AbsoluteX
			| AddressingMode::AbsoluteY
			| AddressingMode::Indirect => {
				let value = Self::extract_word_operand(operand, pos)?;
				bytes.extend_from_slice(&value.to_le_bytes());
			}
		}

		Ok(bytes)
	}

	/// Extract a byte operand from the operand
	fn extract_byte_operand(operand: &Operand, pos: &SourcePos) -> AssemblyResult<u8> {
		match operand {
			Operand::Immediate8(val) => Ok(*val),
			Operand::ZeroPage(val) => Ok(*val),
			Operand::Immediate16(val) => {
				if *val > 0xFF {
					Err(AssemblyError::invalid_address(
						pos.clone(),
						*val,
						"Value too large for 8-bit operand".to_string(),
					))
				} else {
					Ok(*val as u8)
				}
			}
			Operand::Absolute(val) => {
				if *val > 0xFF {
					Err(AssemblyError::invalid_address(
						pos.clone(),
						*val,
						"Value too large for 8-bit operand".to_string(),
					))
				} else {
					Ok(*val as u8)
				}
			}
			Operand::Symbol(_) | Operand::Expression(_) => {
				Err(AssemblyError::symbol(pos.clone(), "Unresolved symbol in operand".to_string()))
			}
			Operand::None => Err(AssemblyError::parse(
				pos.clone(),
				"Expected operand but none provided".to_string(),
			)),
			Operand::Relative(_) => Err(AssemblyError::parse(
				pos.clone(),
				"Relative operand not valid for this addressing mode".to_string(),
			)),
		}
	}

	/// Extract a word operand from the operand
	fn extract_word_operand(operand: &Operand, pos: &SourcePos) -> AssemblyResult<u16> {
		match operand {
			Operand::Absolute(val) => Ok(*val),
			Operand::Immediate16(val) => Ok(*val),
			Operand::Immediate8(val) => Ok(*val as u16),
			Operand::ZeroPage(val) => Ok(*val as u16),
			Operand::Symbol(_) | Operand::Expression(_) => {
				Err(AssemblyError::symbol(pos.clone(), "Unresolved symbol in operand".to_string()))
			}
			Operand::None => Err(AssemblyError::parse(
				pos.clone(),
				"Expected operand but none provided".to_string(),
			)),
			Operand::Relative(_) => Err(AssemblyError::parse(
				pos.clone(),
				"Relative operand not valid for this addressing mode".to_string(),
			)),
		}
	}

	/// Extract a relative operand from the operand
	fn extract_relative_operand(operand: &Operand, pos: &SourcePos) -> AssemblyResult<i8> {
		match operand {
			Operand::Relative(offset) => Ok(*offset),
			Operand::Immediate8(val) => {
				let signed_val = *val as i8;
				Ok(signed_val)
			}
			Operand::Symbol(_) | Operand::Expression(_) => Err(AssemblyError::symbol(
				pos.clone(),
				"Unresolved symbol in relative operand".to_string(),
			)),
			_ => Err(AssemblyError::parse(
				pos.clone(),
				"Invalid operand type for relative addressing".to_string(),
			)),
		}
	}

	/// Calculate relative offset between two addresses
	pub fn calculate_relative_offset(from: u16, to: u16, pos: &SourcePos) -> AssemblyResult<i8> {
		// The relative offset is calculated from the address of the next instruction
		// (current PC + 2 for branch instructions)
		let next_pc = from.wrapping_add(2);
		let offset = to.wrapping_sub(next_pc) as i16;

		if offset < -128 || offset > 127 {
			Err(AssemblyError::invalid_address(
				pos.clone(),
				to,
				format!(
					"Branch target ${:04X} is too far from ${:04X} (offset: {})",
					to, from, offset
				),
			))
		} else {
			Ok(offset as i8)
		}
	}

	/// Validate that an operand is appropriate for an addressing mode
	pub fn validate_operand(
		addressing_mode: AddressingMode,
		operand: &Operand,
		pos: &SourcePos,
	) -> AssemblyResult<()> {
		match (addressing_mode, operand) {
			(AddressingMode::Implied | AddressingMode::Accumulator, Operand::None) => Ok(()),

			(AddressingMode::Immediate, Operand::Immediate8(_)) => Ok(()),
			(AddressingMode::Immediate, Operand::Immediate16(val)) if *val <= 0xFF => Ok(()),

			(AddressingMode::ZeroPage, Operand::ZeroPage(_)) => Ok(()),
			(AddressingMode::ZeroPage, Operand::Absolute(val)) if *val <= 0xFF => Ok(()),

			(AddressingMode::ZeroPageX, Operand::ZeroPage(_)) => Ok(()),
			(AddressingMode::ZeroPageX, Operand::Absolute(val)) if *val <= 0xFF => Ok(()),

			(AddressingMode::ZeroPageY, Operand::ZeroPage(_)) => Ok(()),
			(AddressingMode::ZeroPageY, Operand::Absolute(val)) if *val <= 0xFF => Ok(()),

			(
				AddressingMode::Absolute
				| AddressingMode::AbsoluteX
				| AddressingMode::AbsoluteY
				| AddressingMode::Indirect,
				Operand::Absolute(_),
			) => Ok(()),
			(
				AddressingMode::Absolute
				| AddressingMode::AbsoluteX
				| AddressingMode::AbsoluteY
				| AddressingMode::Indirect,
				Operand::Immediate16(_),
			) => Ok(()),

			(
				AddressingMode::IndexedIndirect | AddressingMode::IndirectIndexed,
				Operand::ZeroPage(_),
			) => Ok(()),
			(
				AddressingMode::IndexedIndirect | AddressingMode::IndirectIndexed,
				Operand::Absolute(val),
			) if *val <= 0xFF => Ok(()),

			(AddressingMode::Relative, Operand::Relative(_)) => Ok(()),

			// Symbol and expression operands are always valid (resolved later)
			(_, Operand::Symbol(_) | Operand::Expression(_)) => Ok(()),

			_ => Err(AssemblyError::InvalidInstruction {
				pos: pos.clone(),
				message: format!(
					"Operand {:?} is not valid for addressing mode {}",
					operand, addressing_mode
				),
			}),
		}
	}
}

/// Convenience function for encoding instructions
pub fn encode_instruction(
	mnemonic: Mnemonic,
	addressing_mode: AddressingMode,
	operand: &Operand,
	pos: &SourcePos,
) -> AssemblyResult<Vec<u8>> {
	// First validate the operand
	InstructionEncoder::validate_operand(addressing_mode, operand, pos)?;

	// Then encode the instruction
	InstructionEncoder::encode(mnemonic, addressing_mode, operand, pos)
}

/// Encoding context for multi-pass assembly
#[derive(Debug, Clone)]
pub struct EncodingContext {
	/// Current program counter
	pub pc: u16,
	/// Current pass number
	pub pass: usize,
	/// Whether we're in the final pass
	pub final_pass: bool,
}

impl EncodingContext {
	/// Create a new encoding context
	pub fn new(pc: u16, pass: usize, final_pass: bool) -> Self {
		Self {
			pc,
			pass,
			final_pass,
		}
	}

	/// Advance the program counter
	pub fn advance(&mut self, bytes: usize) {
		self.pc = self.pc.wrapping_add(bytes as u16);
	}

	/// Calculate relative branch target
	pub fn calculate_branch_offset(&self, target: u16, pos: &SourcePos) -> AssemblyResult<i8> {
		InstructionEncoder::calculate_relative_offset(self.pc, target, pos)
	}
}

/// Instruction size calculator
pub struct InstructionSizer;

impl InstructionSizer {
	/// Get the size of an instruction in bytes
	pub fn instruction_size(
		mnemonic: Mnemonic,
		addressing_mode: AddressingMode,
		pos: &SourcePos,
	) -> AssemblyResult<usize> {
		// Verify the instruction/addressing mode combination is valid
		let opcode_info = get_opcode(mnemonic, addressing_mode).ok_or_else(|| {
			AssemblyError::InvalidInstruction {
				pos: pos.clone(),
				message: format!(
					"Invalid addressing mode {} for instruction {}",
					addressing_mode, mnemonic
				),
			}
		})?;

		Ok(opcode_info.size())
	}

	/// Get the size of an operand for a given addressing mode
	pub fn operand_size(addressing_mode: AddressingMode) -> usize {
		addressing_mode.operand_size()
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
	fn test_encode_implied_instruction() {
		let pos = test_pos();
		let bytes =
			encode_instruction(Mnemonic::Nop, AddressingMode::Implied, &Operand::None, &pos)
				.unwrap();

		assert_eq!(bytes, vec![0xEA]);
	}

	#[test]
	fn test_encode_immediate_instruction() {
		let pos = test_pos();
		let bytes = encode_instruction(
			Mnemonic::Lda,
			AddressingMode::Immediate,
			&Operand::Immediate8(0x42),
			&pos,
		)
		.unwrap();

		assert_eq!(bytes, vec![0xA9, 0x42]);
	}

	#[test]
	fn test_encode_absolute_instruction() {
		let pos = test_pos();
		let bytes = encode_instruction(
			Mnemonic::Lda,
			AddressingMode::Absolute,
			&Operand::Absolute(0x1234),
			&pos,
		)
		.unwrap();

		assert_eq!(bytes, vec![0xAD, 0x34, 0x12]); // Little-endian
	}

	#[test]
	fn test_encode_zero_page_instruction() {
		let pos = test_pos();
		let bytes = encode_instruction(
			Mnemonic::Lda,
			AddressingMode::ZeroPage,
			&Operand::ZeroPage(0x80),
			&pos,
		)
		.unwrap();

		assert_eq!(bytes, vec![0xA5, 0x80]);
	}

	#[test]
	fn test_encode_relative_instruction() {
		let pos = test_pos();
		let bytes = encode_instruction(
			Mnemonic::Bne,
			AddressingMode::Relative,
			&Operand::Relative(10),
			&pos,
		)
		.unwrap();

		assert_eq!(bytes, vec![0xD0, 10]);
	}

	#[test]
	fn test_encode_negative_relative() {
		let pos = test_pos();
		let bytes = encode_instruction(
			Mnemonic::Bne,
			AddressingMode::Relative,
			&Operand::Relative(-10),
			&pos,
		)
		.unwrap();

		assert_eq!(bytes, vec![0xD0, 0xF6]); // -10 as unsigned byte
	}

	#[test]
	fn test_invalid_instruction_addressing_mode() {
		let pos = test_pos();
		let result = encode_instruction(
			Mnemonic::Lda,
			AddressingMode::Relative, // Invalid for LDA
			&Operand::Relative(10),
			&pos,
		);

		assert!(result.is_err());
	}

	#[test]
	fn test_calculate_relative_offset() {
		let pos = test_pos();

		// Forward branch
		let offset = InstructionEncoder::calculate_relative_offset(0x8000, 0x8010, &pos).unwrap();
		assert_eq!(offset, 14); // 0x8010 - (0x8000 + 2) = 14

		// Backward branch
		let offset = InstructionEncoder::calculate_relative_offset(0x8010, 0x8000, &pos).unwrap();
		assert_eq!(offset, -18); // 0x8000 - (0x8010 + 2) = -18
	}

	#[test]
	fn test_relative_offset_out_of_range() {
		let pos = test_pos();

		// Too far forward
		let result = InstructionEncoder::calculate_relative_offset(0x8000, 0x8200, &pos);
		assert!(result.is_err());

		// Too far backward
		let result = InstructionEncoder::calculate_relative_offset(0x8200, 0x8000, &pos);
		assert!(result.is_err());
	}

	#[test]
	fn test_validate_operand() {
		let pos = test_pos();

		// Valid combinations
		assert!(
			InstructionEncoder::validate_operand(
				AddressingMode::Immediate,
				&Operand::Immediate8(0x42),
				&pos
			)
			.is_ok()
		);

		assert!(
			InstructionEncoder::validate_operand(
				AddressingMode::ZeroPage,
				&Operand::ZeroPage(0x80),
				&pos
			)
			.is_ok()
		);

		assert!(
			InstructionEncoder::validate_operand(
				AddressingMode::Absolute,
				&Operand::Absolute(0x1234),
				&pos
			)
			.is_ok()
		);

		// Invalid combinations
		assert!(
			InstructionEncoder::validate_operand(
				AddressingMode::Immediate,
				&Operand::Absolute(0x1234),
				&pos
			)
			.is_err()
		);

		assert!(
			InstructionEncoder::validate_operand(
				AddressingMode::ZeroPage,
				&Operand::Absolute(0x1234), // Too large for zero page
				&pos
			)
			.is_err()
		);
	}

	#[test]
	fn test_encoding_context() {
		let mut ctx = EncodingContext::new(0x8000, 1, false);
		assert_eq!(ctx.pc, 0x8000);
		assert_eq!(ctx.pass, 1);
		assert!(!ctx.final_pass);

		ctx.advance(3);
		assert_eq!(ctx.pc, 0x8003);

		let pos = test_pos();
		let offset = ctx.calculate_branch_offset(0x8010, &pos).unwrap();
		assert_eq!(offset, 11); // 0x8010 - (0x8003 + 2) = 11
	}

	#[test]
	fn test_instruction_sizer() {
		let pos = test_pos();

		assert_eq!(
			InstructionSizer::instruction_size(Mnemonic::Nop, AddressingMode::Implied, &pos)
				.unwrap(),
			1
		);

		assert_eq!(
			InstructionSizer::instruction_size(Mnemonic::Lda, AddressingMode::Immediate, &pos)
				.unwrap(),
			2
		);

		assert_eq!(
			InstructionSizer::instruction_size(Mnemonic::Lda, AddressingMode::Absolute, &pos)
				.unwrap(),
			3
		);

		// Invalid combination
		assert!(
			InstructionSizer::instruction_size(Mnemonic::Lda, AddressingMode::Relative, &pos)
				.is_err()
		);
	}

	#[test]
	fn test_operand_sizes() {
		assert_eq!(InstructionSizer::operand_size(AddressingMode::Implied), 0);
		assert_eq!(InstructionSizer::operand_size(AddressingMode::Immediate), 1);
		assert_eq!(InstructionSizer::operand_size(AddressingMode::ZeroPage), 1);
		assert_eq!(InstructionSizer::operand_size(AddressingMode::Absolute), 2);
		assert_eq!(InstructionSizer::operand_size(AddressingMode::Relative), 1);
	}
}
