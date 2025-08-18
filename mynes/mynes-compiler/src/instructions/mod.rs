//! Instruction system for the NES assembler.
//!
//! This module provides comprehensive 6502 instruction support including
//! addressing modes, instruction encoding, and NES-specific extensions.

pub mod addressing;
pub mod encoding;
pub mod opcodes;
pub mod validation;

// Re-exports for convenience
pub use addressing::{AddressingMode, AddressingModeType};
pub use encoding::{InstructionEncoder, encode_instruction};
pub use opcodes::{Instruction, Mnemonic, OpcodeInfo};
pub use validation::{InstructionValidator, validate_instruction};

use crate::error::{AssemblyError, AssemblyResult, SourcePos};

/// Standard 6502 instruction size limits
pub const MAX_INSTRUCTION_SIZE: usize = 3;
pub const MIN_INSTRUCTION_SIZE: usize = 1;

/// Instruction operand value
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operand {
	/// No operand (implied)
	None,
	/// 8-bit immediate value
	Immediate8(u8),
	/// 16-bit immediate value
	Immediate16(u16),
	/// 8-bit zero page address
	ZeroPage(u8),
	/// 16-bit absolute address
	Absolute(u16),
	/// Relative branch offset
	Relative(i8),
	/// Symbol reference
	Symbol(String),
	/// Expression to be evaluated
	Expression(String),
}

impl Operand {
	/// Check if this operand requires a symbol resolution
	pub fn needs_resolution(&self) -> bool {
		matches!(self, Self::Symbol(_) | Self::Expression(_))
	}

	/// Get the size in bytes of this operand
	pub fn size(&self) -> usize {
		match self {
			Self::None => 0,
			Self::Immediate8(_) | Self::ZeroPage(_) | Self::Relative(_) => 1,
			Self::Immediate16(_) | Self::Absolute(_) => 2,
			Self::Symbol(_) | Self::Expression(_) => 2, // Assume 16-bit by default
		}
	}

	/// Convert to bytes for encoding
	pub fn to_bytes(&self) -> Vec<u8> {
		match self {
			Self::None => vec![],
			Self::Immediate8(val) | Self::ZeroPage(val) => vec![*val],
			Self::Relative(val) => vec![*val as u8],
			Self::Immediate16(val) | Self::Absolute(val) => val.to_le_bytes().to_vec(),
			Self::Symbol(_) | Self::Expression(_) => {
				// These should be resolved before encoding
				vec![0x00, 0x00]
			}
		}
	}
}

/// Complete instruction with mnemonic, addressing mode, and operand
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteInstruction {
	/// Instruction mnemonic
	pub mnemonic: Mnemonic,
	/// Addressing mode
	pub addressing_mode: AddressingMode,
	/// Operand value
	pub operand: Operand,
	/// Source position for error reporting
	pub source_pos: SourcePos,
}

impl CompleteInstruction {
	/// Create a new complete instruction
	pub fn new(
		mnemonic: Mnemonic,
		addressing_mode: AddressingMode,
		operand: Operand,
		source_pos: SourcePos,
	) -> Self {
		Self {
			mnemonic,
			addressing_mode,
			operand,
			source_pos,
		}
	}

	/// Get the total size of this instruction in bytes
	pub fn size(&self) -> usize {
		1 + self.operand.size() // Opcode + operand
	}

	/// Check if this instruction needs symbol resolution
	pub fn needs_resolution(&self) -> bool {
		self.operand.needs_resolution()
	}

	/// Validate this instruction
	pub fn validate(&self) -> AssemblyResult<()> {
		validate_instruction(self.mnemonic, self.addressing_mode, &self.source_pos)
	}

	/// Encode this instruction to bytes
	pub fn encode(&self) -> AssemblyResult<Vec<u8>> {
		encode_instruction(self.mnemonic, self.addressing_mode, &self.operand, &self.source_pos)
	}
}

/// Instruction builder for fluent API
#[derive(Debug)]
pub struct InstructionBuilder {
	mnemonic: Option<Mnemonic>,
	addressing_mode: Option<AddressingMode>,
	operand: Option<Operand>,
	source_pos: Option<SourcePos>,
}

impl InstructionBuilder {
	/// Create a new instruction builder
	pub fn new() -> Self {
		Self {
			mnemonic: None,
			addressing_mode: None,
			operand: None,
			source_pos: None,
		}
	}

	/// Set the mnemonic
	pub fn mnemonic(mut self, mnemonic: Mnemonic) -> Self {
		self.mnemonic = Some(mnemonic);
		self
	}

	/// Set the addressing mode
	pub fn addressing_mode(mut self, mode: AddressingMode) -> Self {
		self.addressing_mode = Some(mode);
		self
	}

	/// Set the operand
	pub fn operand(mut self, operand: Operand) -> Self {
		self.operand = Some(operand);
		self
	}

	/// Set the source position
	pub fn source_pos(mut self, pos: SourcePos) -> Self {
		self.source_pos = Some(pos);
		self
	}

	/// Build the complete instruction
	pub fn build(self) -> AssemblyResult<CompleteInstruction> {
		let mnemonic = self.mnemonic.ok_or_else(|| {
			AssemblyError::internal(None, "Missing mnemonic in instruction builder".to_string())
		})?;

		let addressing_mode = self.addressing_mode.ok_or_else(|| {
			AssemblyError::internal(
				None,
				"Missing addressing mode in instruction builder".to_string(),
			)
		})?;

		let operand = self.operand.unwrap_or(Operand::None);

		let source_pos = self.source_pos.ok_or_else(|| {
			AssemblyError::internal(
				None,
				"Missing source position in instruction builder".to_string(),
			)
		})?;

		let instruction = CompleteInstruction::new(mnemonic, addressing_mode, operand, source_pos);
		instruction.validate()?;
		Ok(instruction)
	}
}

impl Default for InstructionBuilder {
	fn default() -> Self {
		Self::new()
	}
}

/// Instruction factory for common instruction patterns
pub struct InstructionFactory;

impl InstructionFactory {
	/// Create an implied instruction (no operand)
	pub fn implied(mnemonic: Mnemonic, pos: SourcePos) -> AssemblyResult<CompleteInstruction> {
		InstructionBuilder::new()
			.mnemonic(mnemonic)
			.addressing_mode(AddressingMode::Implied)
			.operand(Operand::None)
			.source_pos(pos)
			.build()
	}

	/// Create an immediate instruction
	pub fn immediate(
		mnemonic: Mnemonic,
		value: u8,
		pos: SourcePos,
	) -> AssemblyResult<CompleteInstruction> {
		InstructionBuilder::new()
			.mnemonic(mnemonic)
			.addressing_mode(AddressingMode::Immediate)
			.operand(Operand::Immediate8(value))
			.source_pos(pos)
			.build()
	}

	/// Create a zero page instruction
	pub fn zero_page(
		mnemonic: Mnemonic,
		address: u8,
		pos: SourcePos,
	) -> AssemblyResult<CompleteInstruction> {
		InstructionBuilder::new()
			.mnemonic(mnemonic)
			.addressing_mode(AddressingMode::ZeroPage)
			.operand(Operand::ZeroPage(address))
			.source_pos(pos)
			.build()
	}

	/// Create an absolute instruction
	pub fn absolute(
		mnemonic: Mnemonic,
		address: u16,
		pos: SourcePos,
	) -> AssemblyResult<CompleteInstruction> {
		InstructionBuilder::new()
			.mnemonic(mnemonic)
			.addressing_mode(AddressingMode::Absolute)
			.operand(Operand::Absolute(address))
			.source_pos(pos)
			.build()
	}

	/// Create a relative branch instruction
	pub fn relative(
		mnemonic: Mnemonic,
		offset: i8,
		pos: SourcePos,
	) -> AssemblyResult<CompleteInstruction> {
		InstructionBuilder::new()
			.mnemonic(mnemonic)
			.addressing_mode(AddressingMode::Relative)
			.operand(Operand::Relative(offset))
			.source_pos(pos)
			.build()
	}

	/// Create an instruction with a symbol operand
	pub fn with_symbol(
		mnemonic: Mnemonic,
		addressing_mode: AddressingMode,
		symbol: String,
		pos: SourcePos,
	) -> AssemblyResult<CompleteInstruction> {
		InstructionBuilder::new()
			.mnemonic(mnemonic)
			.addressing_mode(addressing_mode)
			.operand(Operand::Symbol(symbol))
			.source_pos(pos)
			.build()
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
	fn test_operand_size() {
		assert_eq!(Operand::None.size(), 0);
		assert_eq!(Operand::Immediate8(0x42).size(), 1);
		assert_eq!(Operand::ZeroPage(0x80).size(), 1);
		assert_eq!(Operand::Immediate16(0x1234).size(), 2);
		assert_eq!(Operand::Absolute(0x8000).size(), 2);
		assert_eq!(Operand::Relative(-10).size(), 1);
		assert_eq!(Operand::Symbol("test".to_string()).size(), 2);
	}

	#[test]
	fn test_operand_to_bytes() {
		assert_eq!(Operand::None.to_bytes(), Vec::<u8>::new());
		assert_eq!(Operand::Immediate8(0x42).to_bytes(), vec![0x42]);
		assert_eq!(Operand::ZeroPage(0x80).to_bytes(), vec![0x80]);
		assert_eq!(Operand::Immediate16(0x1234).to_bytes(), vec![0x34, 0x12]);
		assert_eq!(Operand::Absolute(0x8000).to_bytes(), vec![0x00, 0x80]);
		assert_eq!(Operand::Relative(-1).to_bytes(), vec![0xFF]);
	}

	#[test]
	fn test_operand_needs_resolution() {
		assert!(!Operand::None.needs_resolution());
		assert!(!Operand::Immediate8(0x42).needs_resolution());
		assert!(Operand::Symbol("test".to_string()).needs_resolution());
		assert!(Operand::Expression("2+2".to_string()).needs_resolution());
	}

	#[test]
	fn test_instruction_builder() {
		let pos = test_pos();
		let instruction = InstructionBuilder::new()
			.mnemonic(Mnemonic::Lda)
			.addressing_mode(AddressingMode::Immediate)
			.operand(Operand::Immediate8(0x42))
			.source_pos(pos.clone())
			.build();

		assert!(instruction.is_ok());
		let inst = instruction.unwrap();
		assert_eq!(inst.mnemonic, Mnemonic::Lda);
		assert_eq!(inst.addressing_mode, AddressingMode::Immediate);
		assert_eq!(inst.operand, Operand::Immediate8(0x42));
		assert_eq!(inst.size(), 2); // Opcode + 1 byte operand
	}

	#[test]
	fn test_instruction_factory() {
		let pos = test_pos();

		// Test implied instruction
		let nop = InstructionFactory::implied(Mnemonic::Nop, pos.clone()).unwrap();
		assert_eq!(nop.mnemonic, Mnemonic::Nop);
		assert_eq!(nop.addressing_mode, AddressingMode::Implied);
		assert_eq!(nop.size(), 1);

		// Test immediate instruction
		let lda = InstructionFactory::immediate(Mnemonic::Lda, 0x42, pos.clone()).unwrap();
		assert_eq!(lda.mnemonic, Mnemonic::Lda);
		assert_eq!(lda.addressing_mode, AddressingMode::Immediate);
		assert_eq!(lda.operand, Operand::Immediate8(0x42));
		assert_eq!(lda.size(), 2);

		// Test absolute instruction
		let jmp = InstructionFactory::absolute(Mnemonic::Jmp, 0x8000, pos).unwrap();
		assert_eq!(jmp.mnemonic, Mnemonic::Jmp);
		assert_eq!(jmp.addressing_mode, AddressingMode::Absolute);
		assert_eq!(jmp.operand, Operand::Absolute(0x8000));
		assert_eq!(jmp.size(), 3);
	}

	#[test]
	fn test_complete_instruction() {
		let pos = test_pos();
		let instruction = CompleteInstruction::new(
			Mnemonic::Lda,
			AddressingMode::Immediate,
			Operand::Immediate8(0x42),
			pos,
		);

		assert_eq!(instruction.size(), 2);
		assert!(!instruction.needs_resolution());
	}

	#[test]
	fn test_instruction_with_symbol() {
		let pos = test_pos();
		let instruction = CompleteInstruction::new(
			Mnemonic::Jmp,
			AddressingMode::Absolute,
			Operand::Symbol("main_loop".to_string()),
			pos,
		);

		assert_eq!(instruction.size(), 3);
		assert!(instruction.needs_resolution());
	}
}
