//! Instruction validation for 6502 instructions.
//!
//! This module provides comprehensive validation of 6502 instructions,
//! including addressing mode compatibility, operand validation, and
//! instruction-specific constraints.

use crate::error::{AssemblyError, AssemblyResult, SourcePos};
use crate::instructions::addressing::AddressingMode;
use crate::instructions::opcodes::{InstructionCategory, Mnemonic, get_instruction};

/// Instruction validator for checking instruction validity
pub struct InstructionValidator;

impl InstructionValidator {
	/// Validate an instruction with its addressing mode
	pub fn validate(
		mnemonic: Mnemonic,
		addressing_mode: AddressingMode,
		pos: &SourcePos,
	) -> AssemblyResult<()> {
		// Check if the instruction exists
		let instruction =
			get_instruction(mnemonic).ok_or_else(|| AssemblyError::InvalidInstruction {
				pos: pos.clone(),
				message: format!("Unknown instruction: {}", mnemonic),
			})?;

		// Check if the addressing mode is supported
		if !instruction.supports_addressing_mode(addressing_mode) {
			return Err(AssemblyError::InvalidInstruction {
				pos: pos.clone(),
				message: format!(
					"Addressing mode {} not supported for instruction {}",
					addressing_mode, mnemonic
				),
			});
		}

		// Perform instruction-specific validation
		Self::validate_instruction_specific(mnemonic, addressing_mode, pos)?;

		Ok(())
	}

	/// Perform instruction-specific validation rules
	fn validate_instruction_specific(
		mnemonic: Mnemonic,
		addressing_mode: AddressingMode,
		pos: &SourcePos,
	) -> AssemblyResult<()> {
		match mnemonic {
			// Branch instructions can only use relative addressing
			Mnemonic::Bcc
			| Mnemonic::Bcs
			| Mnemonic::Beq
			| Mnemonic::Bmi
			| Mnemonic::Bne
			| Mnemonic::Bpl
			| Mnemonic::Bvc
			| Mnemonic::Bvs => {
				if addressing_mode != AddressingMode::Relative {
					return Err(AssemblyError::InvalidInstruction {
						pos: pos.clone(),
						message: format!(
							"Branch instruction {} requires relative addressing",
							mnemonic
						),
					});
				}
			}

			// Jump instructions have specific addressing mode constraints
			Mnemonic::Jmp => {
				if !matches!(addressing_mode, AddressingMode::Absolute | AddressingMode::Indirect) {
					return Err(AssemblyError::InvalidInstruction {
						pos: pos.clone(),
						message: "JMP instruction requires absolute or indirect addressing"
							.to_string(),
					});
				}
			}

			// JSR can only use absolute addressing
			Mnemonic::Jsr => {
				if addressing_mode != AddressingMode::Absolute {
					return Err(AssemblyError::InvalidInstruction {
						pos: pos.clone(),
						message: "JSR instruction requires absolute addressing".to_string(),
					});
				}
			}

			// Implied instructions must use implied addressing
			Mnemonic::Tax
			| Mnemonic::Tay
			| Mnemonic::Txa
			| Mnemonic::Tya
			| Mnemonic::Tsx
			| Mnemonic::Txs
			| Mnemonic::Pha
			| Mnemonic::Php
			| Mnemonic::Pla
			| Mnemonic::Plp
			| Mnemonic::Inx
			| Mnemonic::Iny
			| Mnemonic::Dex
			| Mnemonic::Dey
			| Mnemonic::Clc
			| Mnemonic::Cld
			| Mnemonic::Cli
			| Mnemonic::Clv
			| Mnemonic::Sec
			| Mnemonic::Sed
			| Mnemonic::Sei
			| Mnemonic::Nop
			| Mnemonic::Brk
			| Mnemonic::Rti
			| Mnemonic::Rts => {
				if addressing_mode != AddressingMode::Implied {
					return Err(AssemblyError::InvalidInstruction {
						pos: pos.clone(),
						message: format!("Instruction {} requires implied addressing", mnemonic),
					});
				}
			}

			// Store instructions cannot use immediate addressing
			Mnemonic::Sta | Mnemonic::Stx | Mnemonic::Sty => {
				if addressing_mode == AddressingMode::Immediate {
					return Err(AssemblyError::InvalidInstruction {
						pos: pos.clone(),
						message: format!(
							"Store instruction {} cannot use immediate addressing",
							mnemonic
						),
					});
				}
			}

			// Compare with X/Y have limited addressing modes
			Mnemonic::Cpx | Mnemonic::Cpy => {
				if !matches!(
					addressing_mode,
					AddressingMode::Immediate | AddressingMode::ZeroPage | AddressingMode::Absolute
				) {
					return Err(AssemblyError::InvalidInstruction {
						pos: pos.clone(),
						message: format!(
							"Instruction {} only supports immediate, zero page, or absolute addressing",
							mnemonic
						),
					});
				}
			}

			// BIT instruction has limited addressing modes
			Mnemonic::Bit => {
				if !matches!(addressing_mode, AddressingMode::ZeroPage | AddressingMode::Absolute) {
					return Err(AssemblyError::InvalidInstruction {
						pos: pos.clone(),
						message: "BIT instruction only supports zero page or absolute addressing"
							.to_string(),
					});
				}
			}

			// Shift/rotate instructions can use accumulator or memory addressing
			Mnemonic::Asl | Mnemonic::Lsr | Mnemonic::Rol | Mnemonic::Ror => {
				match addressing_mode {
					AddressingMode::Accumulator
					| AddressingMode::ZeroPage
					| AddressingMode::ZeroPageX
					| AddressingMode::Absolute
					| AddressingMode::AbsoluteX => {
						// Valid addressing modes
					}
					_ => {
						return Err(AssemblyError::InvalidInstruction {
							pos: pos.clone(),
							message: format!(
								"Shift/rotate instruction {} does not support {} addressing",
								mnemonic, addressing_mode
							),
						});
					}
				}
			}

			// INC/DEC memory instructions cannot use immediate or accumulator
			Mnemonic::Inc | Mnemonic::Dec => {
				if matches!(
					addressing_mode,
					AddressingMode::Immediate | AddressingMode::Accumulator
				) {
					return Err(AssemblyError::InvalidInstruction {
						pos: pos.clone(),
						message: format!(
							"Instruction {} cannot use immediate or accumulator addressing",
							mnemonic
						),
					});
				}
			}

			// Other instructions are validated by the opcode table lookup
			_ => {}
		}

		Ok(())
	}

	/// Validate that an instruction is appropriate for the target platform
	pub fn validate_platform_compatibility(
		mnemonic: Mnemonic,
		allow_unofficial: bool,
		pos: &SourcePos,
	) -> AssemblyResult<()> {
		if mnemonic.is_unofficial() && !allow_unofficial {
			return Err(AssemblyError::InvalidInstruction {
				pos: pos.clone(),
				message: format!(
					"Unofficial instruction {} not allowed (use --allow-unofficial to enable)",
					mnemonic
				),
			});
		}

		Ok(())
	}

	/// Check if an addressing mode is valid for a given operand size
	pub fn validate_operand_size(
		addressing_mode: AddressingMode,
		operand_value: Option<u16>,
		pos: &SourcePos,
	) -> AssemblyResult<()> {
		if let Some(value) = operand_value {
			match addressing_mode {
				AddressingMode::ZeroPage
				| AddressingMode::ZeroPageX
				| AddressingMode::ZeroPageY
				| AddressingMode::IndexedIndirect
				| AddressingMode::IndirectIndexed => {
					if value > 0xFF {
						return Err(AssemblyError::invalid_address(
							pos.clone(),
							value,
							format!(
								"Value ${:04X} too large for {} addressing (max $00FF)",
								value, addressing_mode
							),
						));
					}
				}
				AddressingMode::Immediate => {
					if value > 0xFF {
						return Err(AssemblyError::invalid_address(
							pos.clone(),
							value,
							format!("Immediate value ${:04X} too large (max $00FF)", value),
						));
					}
				}
				AddressingMode::Relative => {
					let signed_value = value as i16;
					if signed_value < -128 || signed_value > 127 {
						return Err(AssemblyError::invalid_address(
							pos.clone(),
							value,
							format!("Relative offset {} out of range (-128 to +127)", signed_value),
						));
					}
				}
				_ => {
					// 16-bit addressing modes are always valid for any 16-bit value
				}
			}
		}

		Ok(())
	}

	/// Validate instruction semantics (e.g., branch targets, memory access patterns)
	pub fn validate_semantics(
		mnemonic: Mnemonic,
		addressing_mode: AddressingMode,
		target_address: Option<u16>,
		pos: &SourcePos,
	) -> AssemblyResult<()> {
		// Validate branch targets
		if mnemonic.category() == InstructionCategory::Branch {
			if let Some(target) = target_address {
				// Check if branch target is reasonable
				if target == 0 {
					return Err(AssemblyError::invalid_address(
						pos.clone(),
						target,
						"Branch to address $0000 may be unintentional".to_string(),
					));
				}
			}
		}

		// Validate jump targets
		if matches!(mnemonic, Mnemonic::Jmp | Mnemonic::Jsr) {
			if let Some(target) = target_address {
				// Warn about jumps to zero page (unusual but not necessarily wrong)
				if target < 0x0200 && target != 0 {
					// Note: This could be a warning rather than an error
					// For now, we'll allow it but could add a warning system later
				}

				// Check for jump to interrupt vectors (might be intentional)
				if target >= 0xFFFA {
					// Note: This could be a warning for jumping to interrupt vectors
				}
			}
		}

		// Validate indirect addressing constraints
		if addressing_mode == AddressingMode::Indirect {
			if let Some(target) = target_address {
				// Check for the famous 6502 JMP indirect bug
				if (target & 0xFF) == 0xFF {
					// This could be a warning about the page boundary bug
					// The 6502 has a bug where JMP ($xxFF) reads the high byte from $xx00
					// instead of $xx00 of the next page
				}
			}
		}

		Ok(())
	}

	/// Get instruction complexity score (for optimization hints)
	pub fn complexity_score(mnemonic: Mnemonic, addressing_mode: AddressingMode) -> u8 {
		let base_score = match mnemonic.category() {
			InstructionCategory::LoadStore => 1,
			InstructionCategory::Transfer => 1,
			InstructionCategory::Stack => 2,
			InstructionCategory::Logical => 1,
			InstructionCategory::Arithmetic => 2,
			InstructionCategory::Increment => 1,
			InstructionCategory::Shift => 2,
			InstructionCategory::Jump => 3,
			InstructionCategory::Branch => 2,
			InstructionCategory::Status => 1,
			InstructionCategory::System => 3,
			InstructionCategory::Unofficial => 3,
		};

		let addressing_bonus = match addressing_mode {
			AddressingMode::Implied | AddressingMode::Accumulator => 0,
			AddressingMode::Immediate | AddressingMode::ZeroPage => 0,
			AddressingMode::ZeroPageX | AddressingMode::ZeroPageY => 1,
			AddressingMode::Absolute => 1,
			AddressingMode::AbsoluteX | AddressingMode::AbsoluteY => 2,
			AddressingMode::Indirect => 3,
			AddressingMode::IndexedIndirect | AddressingMode::IndirectIndexed => 3,
			AddressingMode::Relative => 1,
		};

		base_score + addressing_bonus
	}
}

/// Convenience function for validating instructions
pub fn validate_instruction(
	mnemonic: Mnemonic,
	addressing_mode: AddressingMode,
	pos: &SourcePos,
) -> AssemblyResult<()> {
	InstructionValidator::validate(mnemonic, addressing_mode, pos)
}

/// Validate instruction with additional context
pub fn validate_instruction_with_context(
	mnemonic: Mnemonic,
	addressing_mode: AddressingMode,
	operand_value: Option<u16>,
	target_address: Option<u16>,
	allow_unofficial: bool,
	pos: &SourcePos,
) -> AssemblyResult<()> {
	// Basic instruction validation
	InstructionValidator::validate(mnemonic, addressing_mode, pos)?;

	// Platform compatibility
	InstructionValidator::validate_platform_compatibility(mnemonic, allow_unofficial, pos)?;

	// Operand size validation
	InstructionValidator::validate_operand_size(addressing_mode, operand_value, pos)?;

	// Semantic validation
	InstructionValidator::validate_semantics(mnemonic, addressing_mode, target_address, pos)?;

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::path::PathBuf;

	fn test_pos() -> SourcePos {
		SourcePos::new(PathBuf::from("test.asm"), 1, 1)
	}

	#[test]
	fn test_valid_instructions() {
		let pos = test_pos();

		// Valid instruction/addressing mode combinations
		assert!(validate_instruction(Mnemonic::Lda, AddressingMode::Immediate, &pos).is_ok());
		assert!(validate_instruction(Mnemonic::Lda, AddressingMode::ZeroPage, &pos).is_ok());
		assert!(validate_instruction(Mnemonic::Lda, AddressingMode::Absolute, &pos).is_ok());
		assert!(validate_instruction(Mnemonic::Nop, AddressingMode::Implied, &pos).is_ok());
		assert!(validate_instruction(Mnemonic::Jmp, AddressingMode::Absolute, &pos).is_ok());
		assert!(validate_instruction(Mnemonic::Bne, AddressingMode::Relative, &pos).is_ok());
	}

	#[test]
	fn test_invalid_addressing_modes() {
		let pos = test_pos();

		// Invalid instruction/addressing mode combinations
		assert!(validate_instruction(Mnemonic::Lda, AddressingMode::Relative, &pos).is_err());
		assert!(validate_instruction(Mnemonic::Bne, AddressingMode::Absolute, &pos).is_err());
		assert!(validate_instruction(Mnemonic::Nop, AddressingMode::Immediate, &pos).is_err());
		assert!(validate_instruction(Mnemonic::Jsr, AddressingMode::Immediate, &pos).is_err());
	}

	#[test]
	fn test_branch_instruction_validation() {
		let pos = test_pos();

		// Branch instructions must use relative addressing
		assert!(validate_instruction(Mnemonic::Bcc, AddressingMode::Relative, &pos).is_ok());
		assert!(validate_instruction(Mnemonic::Bcs, AddressingMode::Relative, &pos).is_ok());
		assert!(validate_instruction(Mnemonic::Beq, AddressingMode::Relative, &pos).is_ok());
		assert!(validate_instruction(Mnemonic::Bne, AddressingMode::Relative, &pos).is_ok());

		// These should fail
		assert!(validate_instruction(Mnemonic::Bcc, AddressingMode::Absolute, &pos).is_err());
		assert!(validate_instruction(Mnemonic::Beq, AddressingMode::Immediate, &pos).is_err());
	}

	#[test]
	fn test_jump_instruction_validation() {
		let pos = test_pos();

		// JMP supports absolute and indirect
		assert!(validate_instruction(Mnemonic::Jmp, AddressingMode::Absolute, &pos).is_ok());
		assert!(validate_instruction(Mnemonic::Jmp, AddressingMode::Indirect, &pos).is_ok());

		// JSR only supports absolute
		assert!(validate_instruction(Mnemonic::Jsr, AddressingMode::Absolute, &pos).is_ok());

		// These should fail
		assert!(validate_instruction(Mnemonic::Jmp, AddressingMode::Immediate, &pos).is_err());
		assert!(validate_instruction(Mnemonic::Jsr, AddressingMode::Indirect, &pos).is_err());
	}

	#[test]
	fn test_implied_instruction_validation() {
		let pos = test_pos();

		// Implied instructions must use implied addressing
		assert!(validate_instruction(Mnemonic::Tax, AddressingMode::Implied, &pos).is_ok());
		assert!(validate_instruction(Mnemonic::Nop, AddressingMode::Implied, &pos).is_ok());
		assert!(validate_instruction(Mnemonic::Rts, AddressingMode::Implied, &pos).is_ok());

		// These should fail
		assert!(validate_instruction(Mnemonic::Tax, AddressingMode::Immediate, &pos).is_err());
		assert!(validate_instruction(Mnemonic::Nop, AddressingMode::Absolute, &pos).is_err());
	}

	#[test]
	fn test_store_instruction_validation() {
		let pos = test_pos();

		// Store instructions cannot use immediate addressing
		assert!(validate_instruction(Mnemonic::Sta, AddressingMode::ZeroPage, &pos).is_ok());
		assert!(validate_instruction(Mnemonic::Sta, AddressingMode::Absolute, &pos).is_ok());
		assert!(validate_instruction(Mnemonic::Stx, AddressingMode::ZeroPage, &pos).is_ok());

		// These should fail
		assert!(validate_instruction(Mnemonic::Sta, AddressingMode::Immediate, &pos).is_err());
		assert!(validate_instruction(Mnemonic::Stx, AddressingMode::Immediate, &pos).is_err());
	}

	#[test]
	fn test_operand_size_validation() {
		let pos = test_pos();

		// Valid operand sizes
		assert!(
			InstructionValidator::validate_operand_size(AddressingMode::ZeroPage, Some(0x80), &pos)
				.is_ok()
		);
		assert!(
			InstructionValidator::validate_operand_size(
				AddressingMode::Absolute,
				Some(0x1234),
				&pos
			)
			.is_ok()
		);
		assert!(
			InstructionValidator::validate_operand_size(
				AddressingMode::Immediate,
				Some(0xFF),
				&pos
			)
			.is_ok()
		);

		// Invalid operand sizes
		assert!(
			InstructionValidator::validate_operand_size(
				AddressingMode::ZeroPage,
				Some(0x200),
				&pos
			)
			.is_err()
		);
		assert!(
			InstructionValidator::validate_operand_size(
				AddressingMode::Immediate,
				Some(0x200),
				&pos
			)
			.is_err()
		);
		assert!(
			InstructionValidator::validate_operand_size(AddressingMode::Relative, Some(200), &pos)
				.is_err()
		);
	}

	#[test]
	fn test_platform_compatibility() {
		let pos = test_pos();

		// Official instructions should always be allowed
		assert!(
			InstructionValidator::validate_platform_compatibility(Mnemonic::Lda, false, &pos)
				.is_ok()
		);
		assert!(
			InstructionValidator::validate_platform_compatibility(Mnemonic::Lda, true, &pos)
				.is_ok()
		);

		// Unofficial instructions
		assert!(
			InstructionValidator::validate_platform_compatibility(Mnemonic::Lax, true, &pos)
				.is_ok()
		);
		assert!(
			InstructionValidator::validate_platform_compatibility(Mnemonic::Lax, false, &pos)
				.is_err()
		);
	}

	#[test]
	fn test_complexity_scores() {
		// Simple instructions should have low scores
		assert_eq!(
			InstructionValidator::complexity_score(Mnemonic::Lda, AddressingMode::Immediate),
			1
		);
		assert_eq!(
			InstructionValidator::complexity_score(Mnemonic::Tax, AddressingMode::Implied),
			1
		);

		// Complex instructions should have higher scores
		assert_eq!(
			InstructionValidator::complexity_score(Mnemonic::Jmp, AddressingMode::Indirect),
			6
		);
		assert!(
			InstructionValidator::complexity_score(Mnemonic::Lda, AddressingMode::IndexedIndirect)
				> InstructionValidator::complexity_score(Mnemonic::Lda, AddressingMode::Immediate)
		);
	}

	#[test]
	fn test_full_context_validation() {
		let pos = test_pos();

		// Valid instruction with context
		assert!(
			validate_instruction_with_context(
				Mnemonic::Lda,
				AddressingMode::Immediate,
				Some(0x42),
				None,
				false,
				&pos
			)
			.is_ok()
		);

		// Invalid operand size
		assert!(
			validate_instruction_with_context(
				Mnemonic::Lda,
				AddressingMode::ZeroPage,
				Some(0x200),
				None,
				false,
				&pos
			)
			.is_err()
		);

		// Unofficial instruction without permission
		assert!(
			validate_instruction_with_context(
				Mnemonic::Lax,
				AddressingMode::ZeroPage,
				Some(0x80),
				None,
				false,
				&pos
			)
			.is_err()
		);

		// Unofficial instruction with permission
		assert!(
			validate_instruction_with_context(
				Mnemonic::Lax,
				AddressingMode::ZeroPage,
				Some(0x80),
				None,
				true,
				&pos
			)
			.is_ok()
		);
	}
}
