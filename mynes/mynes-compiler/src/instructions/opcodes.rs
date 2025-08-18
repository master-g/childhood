//! 6502 instruction opcodes and mnemonics.
//!
//! This module provides comprehensive definitions of all 6502 instructions,
//! their opcodes, and associated metadata for the NES assembler.

use std::collections::HashMap;
use std::fmt;

use crate::instructions::addressing::AddressingMode;

/// 6502 instruction mnemonics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mnemonic {
	// Load/Store Operations
	/// Load Accumulator
	Lda,
	/// Load X Register
	Ldx,
	/// Load Y Register
	Ldy,
	/// Store Accumulator
	Sta,
	/// Store X Register
	Stx,
	/// Store Y Register
	Sty,

	// Register Transfers
	/// Transfer Accumulator to X
	Tax,
	/// Transfer Accumulator to Y
	Tay,
	/// Transfer X to Accumulator
	Txa,
	/// Transfer Y to Accumulator
	Tya,

	// Stack Operations
	/// Transfer Stack Pointer to X
	Tsx,
	/// Transfer X to Stack Pointer
	Txs,
	/// Push Accumulator
	Pha,
	/// Push Processor Status
	Php,
	/// Pull Accumulator
	Pla,
	/// Pull Processor Status
	Plp,

	// Logical Operations
	/// Logical AND
	And,
	/// Exclusive OR
	Eor,
	/// Logical Inclusive OR
	Ora,
	/// Bit Test
	Bit,

	// Arithmetic Operations
	/// Add with Carry
	Adc,
	/// Subtract with Carry
	Sbc,
	/// Compare
	Cmp,
	/// Compare X Register
	Cpx,
	/// Compare Y Register
	Cpy,

	// Increment/Decrement
	/// Increment Memory
	Inc,
	/// Increment X Register
	Inx,
	/// Increment Y Register
	Iny,
	/// Decrement Memory
	Dec,
	/// Decrement X Register
	Dex,
	/// Decrement Y Register
	Dey,

	// Shifts
	/// Arithmetic Shift Left
	Asl,
	/// Logical Shift Right
	Lsr,
	/// Rotate Left
	Rol,
	/// Rotate Right
	Ror,

	// Jumps & Calls
	/// Jump
	Jmp,
	/// Jump to Subroutine
	Jsr,
	/// Return from Subroutine
	Rts,

	// Branches
	/// Branch if Carry Clear
	Bcc,
	/// Branch if Carry Set
	Bcs,
	/// Branch if Equal (Zero Set)
	Beq,
	/// Branch if Minus (Negative Set)
	Bmi,
	/// Branch if Not Equal (Zero Clear)
	Bne,
	/// Branch if Plus (Negative Clear)
	Bpl,
	/// Branch if Overflow Clear
	Bvc,
	/// Branch if Overflow Set
	Bvs,

	// Status Flag Changes
	/// Clear Carry Flag
	Clc,
	/// Clear Decimal Flag
	Cld,
	/// Clear Interrupt Flag
	Cli,
	/// Clear Overflow Flag
	Clv,
	/// Set Carry Flag
	Sec,
	/// Set Decimal Flag
	Sed,
	/// Set Interrupt Flag
	Sei,

	// System Functions
	/// Break
	Brk,
	/// No Operation
	Nop,
	/// Return from Interrupt
	Rti,

	// Unofficial/Illegal Opcodes (commonly used)
	/// Load Accumulator and X
	Lax,
	/// Store Accumulator AND X
	Sax,
	/// Decrement and Compare
	Dcp,
	/// Increment and Subtract with Carry
	Isc,
	/// Rotate Left and AND
	Rla,
	/// Rotate Right and Add with Carry
	Rra,
	/// Shift Left and OR
	Slo,
	/// Shift Right and EOR
	Sre,
}

impl Mnemonic {
	/// Get all standard (official) mnemonics
	pub fn standard_mnemonics() -> Vec<Self> {
		vec![
			Self::Lda,
			Self::Ldx,
			Self::Ldy,
			Self::Sta,
			Self::Stx,
			Self::Sty,
			Self::Tax,
			Self::Tay,
			Self::Txa,
			Self::Tya,
			Self::Tsx,
			Self::Txs,
			Self::Pha,
			Self::Php,
			Self::Pla,
			Self::Plp,
			Self::And,
			Self::Eor,
			Self::Ora,
			Self::Bit,
			Self::Adc,
			Self::Sbc,
			Self::Cmp,
			Self::Cpx,
			Self::Cpy,
			Self::Inc,
			Self::Inx,
			Self::Iny,
			Self::Dec,
			Self::Dex,
			Self::Dey,
			Self::Asl,
			Self::Lsr,
			Self::Rol,
			Self::Ror,
			Self::Jmp,
			Self::Jsr,
			Self::Rts,
			Self::Bcc,
			Self::Bcs,
			Self::Beq,
			Self::Bmi,
			Self::Bne,
			Self::Bpl,
			Self::Bvc,
			Self::Bvs,
			Self::Clc,
			Self::Cld,
			Self::Cli,
			Self::Clv,
			Self::Sec,
			Self::Sed,
			Self::Sei,
			Self::Brk,
			Self::Nop,
			Self::Rti,
		]
	}

	/// Get all unofficial (illegal) mnemonics
	pub fn unofficial_mnemonics() -> Vec<Self> {
		vec![Self::Lax, Self::Sax, Self::Dcp, Self::Isc, Self::Rla, Self::Rra, Self::Slo, Self::Sre]
	}

	/// Check if this mnemonic is an official 6502 instruction
	pub fn is_official(&self) -> bool {
		Self::standard_mnemonics().contains(self)
	}

	/// Check if this mnemonic is an unofficial/illegal instruction
	pub fn is_unofficial(&self) -> bool {
		Self::unofficial_mnemonics().contains(self)
	}

	/// Get instruction category
	pub fn category(&self) -> InstructionCategory {
		match self {
			Self::Lda | Self::Ldx | Self::Ldy | Self::Sta | Self::Stx | Self::Sty => {
				InstructionCategory::LoadStore
			}
			Self::Tax | Self::Tay | Self::Txa | Self::Tya | Self::Tsx | Self::Txs => {
				InstructionCategory::Transfer
			}
			Self::Pha | Self::Php | Self::Pla | Self::Plp => InstructionCategory::Stack,
			Self::And | Self::Eor | Self::Ora | Self::Bit => InstructionCategory::Logical,
			Self::Adc | Self::Sbc | Self::Cmp | Self::Cpx | Self::Cpy => {
				InstructionCategory::Arithmetic
			}
			Self::Inc | Self::Inx | Self::Iny | Self::Dec | Self::Dex | Self::Dey => {
				InstructionCategory::Increment
			}
			Self::Asl | Self::Lsr | Self::Rol | Self::Ror => InstructionCategory::Shift,
			Self::Jmp | Self::Jsr | Self::Rts => InstructionCategory::Jump,
			Self::Bcc
			| Self::Bcs
			| Self::Beq
			| Self::Bmi
			| Self::Bne
			| Self::Bpl
			| Self::Bvc
			| Self::Bvs => InstructionCategory::Branch,
			Self::Clc | Self::Cld | Self::Cli | Self::Clv | Self::Sec | Self::Sed | Self::Sei => {
				InstructionCategory::Status
			}
			Self::Brk | Self::Nop | Self::Rti => InstructionCategory::System,
			Self::Lax
			| Self::Sax
			| Self::Dcp
			| Self::Isc
			| Self::Rla
			| Self::Rra
			| Self::Slo
			| Self::Sre => InstructionCategory::Unofficial,
		}
	}

	/// Parse a mnemonic from a string
	pub fn parse(s: &str) -> Option<Self> {
		match s.to_uppercase().as_str() {
			"LDA" => Some(Self::Lda),
			"LDX" => Some(Self::Ldx),
			"LDY" => Some(Self::Ldy),
			"STA" => Some(Self::Sta),
			"STX" => Some(Self::Stx),
			"STY" => Some(Self::Sty),
			"TAX" => Some(Self::Tax),
			"TAY" => Some(Self::Tay),
			"TXA" => Some(Self::Txa),
			"TYA" => Some(Self::Tya),
			"TSX" => Some(Self::Tsx),
			"TXS" => Some(Self::Txs),
			"PHA" => Some(Self::Pha),
			"PHP" => Some(Self::Php),
			"PLA" => Some(Self::Pla),
			"PLP" => Some(Self::Plp),
			"AND" => Some(Self::And),
			"EOR" => Some(Self::Eor),
			"ORA" => Some(Self::Ora),
			"BIT" => Some(Self::Bit),
			"ADC" => Some(Self::Adc),
			"SBC" => Some(Self::Sbc),
			"CMP" => Some(Self::Cmp),
			"CPX" => Some(Self::Cpx),
			"CPY" => Some(Self::Cpy),
			"INC" => Some(Self::Inc),
			"INX" => Some(Self::Inx),
			"INY" => Some(Self::Iny),
			"DEC" => Some(Self::Dec),
			"DEX" => Some(Self::Dex),
			"DEY" => Some(Self::Dey),
			"ASL" => Some(Self::Asl),
			"LSR" => Some(Self::Lsr),
			"ROL" => Some(Self::Rol),
			"ROR" => Some(Self::Ror),
			"JMP" => Some(Self::Jmp),
			"JSR" => Some(Self::Jsr),
			"RTS" => Some(Self::Rts),
			"BCC" => Some(Self::Bcc),
			"BCS" => Some(Self::Bcs),
			"BEQ" => Some(Self::Beq),
			"BMI" => Some(Self::Bmi),
			"BNE" => Some(Self::Bne),
			"BPL" => Some(Self::Bpl),
			"BVC" => Some(Self::Bvc),
			"BVS" => Some(Self::Bvs),
			"CLC" => Some(Self::Clc),
			"CLD" => Some(Self::Cld),
			"CLI" => Some(Self::Cli),
			"CLV" => Some(Self::Clv),
			"SEC" => Some(Self::Sec),
			"SED" => Some(Self::Sed),
			"SEI" => Some(Self::Sei),
			"BRK" => Some(Self::Brk),
			"NOP" => Some(Self::Nop),
			"RTI" => Some(Self::Rti),
			// Unofficial opcodes
			"LAX" => Some(Self::Lax),
			"SAX" => Some(Self::Sax),
			"DCP" => Some(Self::Dcp),
			"ISC" | "ISB" => Some(Self::Isc),
			"RLA" => Some(Self::Rla),
			"RRA" => Some(Self::Rra),
			"SLO" => Some(Self::Slo),
			"SRE" => Some(Self::Sre),
			_ => None,
		}
	}
}

impl fmt::Display for Mnemonic {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let name = match self {
			Self::Lda => "LDA",
			Self::Ldx => "LDX",
			Self::Ldy => "LDY",
			Self::Sta => "STA",
			Self::Stx => "STX",
			Self::Sty => "STY",
			Self::Tax => "TAX",
			Self::Tay => "TAY",
			Self::Txa => "TXA",
			Self::Tya => "TYA",
			Self::Tsx => "TSX",
			Self::Txs => "TXS",
			Self::Pha => "PHA",
			Self::Php => "PHP",
			Self::Pla => "PLA",
			Self::Plp => "PLP",
			Self::And => "AND",
			Self::Eor => "EOR",
			Self::Ora => "ORA",
			Self::Bit => "BIT",
			Self::Adc => "ADC",
			Self::Sbc => "SBC",
			Self::Cmp => "CMP",
			Self::Cpx => "CPX",
			Self::Cpy => "CPY",
			Self::Inc => "INC",
			Self::Inx => "INX",
			Self::Iny => "INY",
			Self::Dec => "DEC",
			Self::Dex => "DEX",
			Self::Dey => "DEY",
			Self::Asl => "ASL",
			Self::Lsr => "LSR",
			Self::Rol => "ROL",
			Self::Ror => "ROR",
			Self::Jmp => "JMP",
			Self::Jsr => "JSR",
			Self::Rts => "RTS",
			Self::Bcc => "BCC",
			Self::Bcs => "BCS",
			Self::Beq => "BEQ",
			Self::Bmi => "BMI",
			Self::Bne => "BNE",
			Self::Bpl => "BPL",
			Self::Bvc => "BVC",
			Self::Bvs => "BVS",
			Self::Clc => "CLC",
			Self::Cld => "CLD",
			Self::Cli => "CLI",
			Self::Clv => "CLV",
			Self::Sec => "SEC",
			Self::Sed => "SED",
			Self::Sei => "SEI",
			Self::Brk => "BRK",
			Self::Nop => "NOP",
			Self::Rti => "RTI",
			Self::Lax => "LAX",
			Self::Sax => "SAX",
			Self::Dcp => "DCP",
			Self::Isc => "ISC",
			Self::Rla => "RLA",
			Self::Rra => "RRA",
			Self::Slo => "SLO",
			Self::Sre => "SRE",
		};
		write!(f, "{}", name)
	}
}

/// Instruction categories for organization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionCategory {
	/// Load and store instructions
	LoadStore,
	/// Register transfer instructions
	Transfer,
	/// Stack operations
	Stack,
	/// Logical operations
	Logical,
	/// Arithmetic operations
	Arithmetic,
	/// Increment/decrement operations
	Increment,
	/// Shift and rotate operations
	Shift,
	/// Jump and subroutine instructions
	Jump,
	/// Branch instructions
	Branch,
	/// Status flag instructions
	Status,
	/// System instructions
	System,
	/// Unofficial/illegal instructions
	Unofficial,
}

impl fmt::Display for InstructionCategory {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let name = match self {
			Self::LoadStore => "Load/Store",
			Self::Transfer => "Transfer",
			Self::Stack => "Stack",
			Self::Logical => "Logical",
			Self::Arithmetic => "Arithmetic",
			Self::Increment => "Increment/Decrement",
			Self::Shift => "Shift/Rotate",
			Self::Jump => "Jump/Call",
			Self::Branch => "Branch",
			Self::Status => "Status",
			Self::System => "System",
			Self::Unofficial => "Unofficial",
		};
		write!(f, "{}", name)
	}
}

/// Opcode information for a specific instruction and addressing mode combination
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpcodeInfo {
	/// The opcode byte
	pub opcode: u8,
	/// The addressing mode
	pub addressing_mode: AddressingMode,
	/// Number of cycles (base, without page boundary crossings)
	pub cycles: u8,
	/// Additional cycle if page boundary is crossed
	pub page_cycles: u8,
	/// Instruction affects these processor flags
	pub flags: ProcessorFlags,
}

impl OpcodeInfo {
	/// Create new opcode information
	pub const fn new(
		opcode: u8,
		addressing_mode: AddressingMode,
		cycles: u8,
		page_cycles: u8,
		flags: ProcessorFlags,
	) -> Self {
		Self {
			opcode,
			addressing_mode,
			cycles,
			page_cycles,
			flags,
		}
	}

	/// Get total instruction size in bytes
	pub fn size(&self) -> usize {
		self.addressing_mode.instruction_size()
	}

	/// Calculate effective cycles with page boundary consideration
	pub fn effective_cycles(&self, page_crossed: bool) -> u8 {
		if page_crossed {
			self.cycles + self.page_cycles
		} else {
			self.cycles
		}
	}
}

/// Processor flags that can be affected by instructions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessorFlags {
	/// Affects Negative flag
	pub n: bool,
	/// Affects oVerflow flag
	pub v: bool,
	/// Affects Break flag
	pub b: bool,
	/// Affects Decimal flag
	pub d: bool,
	/// Affects Interrupt flag
	pub i: bool,
	/// Affects Zero flag
	pub z: bool,
	/// Affects Carry flag
	pub c: bool,
}

impl ProcessorFlags {
	/// No flags affected
	pub const NONE: Self = Self {
		n: false,
		v: false,
		b: false,
		d: false,
		i: false,
		z: false,
		c: false,
	};

	/// Standard ALU flags (N, Z)
	pub const NZ: Self = Self {
		n: true,
		v: false,
		b: false,
		d: false,
		i: false,
		z: true,
		c: false,
	};

	/// Arithmetic flags (N, V, Z, C)
	pub const NVZC: Self = Self {
		n: true,
		v: true,
		b: false,
		d: false,
		i: false,
		z: true,
		c: true,
	};

	/// Compare flags (N, Z, C)
	pub const NZC: Self = Self {
		n: true,
		v: false,
		b: false,
		d: false,
		i: false,
		z: true,
		c: true,
	};
}

/// Complete instruction definition
#[derive(Debug, Clone)]
pub struct Instruction {
	/// The mnemonic
	pub mnemonic: Mnemonic,
	/// All valid addressing modes and their opcodes
	pub opcodes: Vec<OpcodeInfo>,
	/// Instruction description
	pub description: &'static str,
}

impl Instruction {
	/// Create a new instruction definition
	pub fn new(mnemonic: Mnemonic, opcodes: Vec<OpcodeInfo>, description: &'static str) -> Self {
		Self {
			mnemonic,
			opcodes,
			description,
		}
	}

	/// Get opcode for a specific addressing mode
	pub fn get_opcode(&self, addressing_mode: AddressingMode) -> Option<&OpcodeInfo> {
		self.opcodes.iter().find(|info| info.addressing_mode == addressing_mode)
	}

	/// Get all supported addressing modes for this instruction
	pub fn addressing_modes(&self) -> Vec<AddressingMode> {
		self.opcodes.iter().map(|info| info.addressing_mode).collect()
	}

	/// Check if this instruction supports a specific addressing mode
	pub fn supports_addressing_mode(&self, addressing_mode: AddressingMode) -> bool {
		self.get_opcode(addressing_mode).is_some()
	}
}

/// Build the complete opcode table for all 6502 instructions
pub fn build_instruction_table() -> HashMap<Mnemonic, Instruction> {
	let mut table = HashMap::new();

	// Load/Store Operations
	table.insert(
		Mnemonic::Lda,
		Instruction::new(
			Mnemonic::Lda,
			vec![
				OpcodeInfo::new(0xA9, AddressingMode::Immediate, 2, 0, ProcessorFlags::NZ),
				OpcodeInfo::new(0xA5, AddressingMode::ZeroPage, 3, 0, ProcessorFlags::NZ),
				OpcodeInfo::new(0xB5, AddressingMode::ZeroPageX, 4, 0, ProcessorFlags::NZ),
				OpcodeInfo::new(0xAD, AddressingMode::Absolute, 4, 0, ProcessorFlags::NZ),
				OpcodeInfo::new(0xBD, AddressingMode::AbsoluteX, 4, 1, ProcessorFlags::NZ),
				OpcodeInfo::new(0xB9, AddressingMode::AbsoluteY, 4, 1, ProcessorFlags::NZ),
				OpcodeInfo::new(0xA1, AddressingMode::IndexedIndirect, 6, 0, ProcessorFlags::NZ),
				OpcodeInfo::new(0xB1, AddressingMode::IndirectIndexed, 5, 1, ProcessorFlags::NZ),
			],
			"Load Accumulator with Memory",
		),
	);

	table.insert(
		Mnemonic::Ldx,
		Instruction::new(
			Mnemonic::Ldx,
			vec![
				OpcodeInfo::new(0xA2, AddressingMode::Immediate, 2, 0, ProcessorFlags::NZ),
				OpcodeInfo::new(0xA6, AddressingMode::ZeroPage, 3, 0, ProcessorFlags::NZ),
				OpcodeInfo::new(0xB6, AddressingMode::ZeroPageY, 4, 0, ProcessorFlags::NZ),
				OpcodeInfo::new(0xAE, AddressingMode::Absolute, 4, 0, ProcessorFlags::NZ),
				OpcodeInfo::new(0xBE, AddressingMode::AbsoluteY, 4, 1, ProcessorFlags::NZ),
			],
			"Load X Register with Memory",
		),
	);

	table.insert(
		Mnemonic::Ldy,
		Instruction::new(
			Mnemonic::Ldy,
			vec![
				OpcodeInfo::new(0xA0, AddressingMode::Immediate, 2, 0, ProcessorFlags::NZ),
				OpcodeInfo::new(0xA4, AddressingMode::ZeroPage, 3, 0, ProcessorFlags::NZ),
				OpcodeInfo::new(0xB4, AddressingMode::ZeroPageX, 4, 0, ProcessorFlags::NZ),
				OpcodeInfo::new(0xAC, AddressingMode::Absolute, 4, 0, ProcessorFlags::NZ),
				OpcodeInfo::new(0xBC, AddressingMode::AbsoluteX, 4, 1, ProcessorFlags::NZ),
			],
			"Load Y Register with Memory",
		),
	);

	table.insert(
		Mnemonic::Sta,
		Instruction::new(
			Mnemonic::Sta,
			vec![
				OpcodeInfo::new(0x85, AddressingMode::ZeroPage, 3, 0, ProcessorFlags::NONE),
				OpcodeInfo::new(0x95, AddressingMode::ZeroPageX, 4, 0, ProcessorFlags::NONE),
				OpcodeInfo::new(0x8D, AddressingMode::Absolute, 4, 0, ProcessorFlags::NONE),
				OpcodeInfo::new(0x9D, AddressingMode::AbsoluteX, 5, 0, ProcessorFlags::NONE),
				OpcodeInfo::new(0x99, AddressingMode::AbsoluteY, 5, 0, ProcessorFlags::NONE),
				OpcodeInfo::new(0x81, AddressingMode::IndexedIndirect, 6, 0, ProcessorFlags::NONE),
				OpcodeInfo::new(0x91, AddressingMode::IndirectIndexed, 6, 0, ProcessorFlags::NONE),
			],
			"Store Accumulator in Memory",
		),
	);

	table.insert(
		Mnemonic::Stx,
		Instruction::new(
			Mnemonic::Stx,
			vec![
				OpcodeInfo::new(0x86, AddressingMode::ZeroPage, 3, 0, ProcessorFlags::NONE),
				OpcodeInfo::new(0x96, AddressingMode::ZeroPageY, 4, 0, ProcessorFlags::NONE),
				OpcodeInfo::new(0x8E, AddressingMode::Absolute, 4, 0, ProcessorFlags::NONE),
			],
			"Store X Register in Memory",
		),
	);

	table.insert(
		Mnemonic::Sty,
		Instruction::new(
			Mnemonic::Sty,
			vec![
				OpcodeInfo::new(0x84, AddressingMode::ZeroPage, 3, 0, ProcessorFlags::NONE),
				OpcodeInfo::new(0x94, AddressingMode::ZeroPageX, 4, 0, ProcessorFlags::NONE),
				OpcodeInfo::new(0x8C, AddressingMode::Absolute, 4, 0, ProcessorFlags::NONE),
			],
			"Store Y Register in Memory",
		),
	);

	// Register Transfers
	table.insert(
		Mnemonic::Tax,
		Instruction::new(
			Mnemonic::Tax,
			vec![OpcodeInfo::new(0xAA, AddressingMode::Implied, 2, 0, ProcessorFlags::NZ)],
			"Transfer Accumulator to X",
		),
	);

	table.insert(
		Mnemonic::Tay,
		Instruction::new(
			Mnemonic::Tay,
			vec![OpcodeInfo::new(0xA8, AddressingMode::Implied, 2, 0, ProcessorFlags::NZ)],
			"Transfer Accumulator to Y",
		),
	);

	table.insert(
		Mnemonic::Txa,
		Instruction::new(
			Mnemonic::Txa,
			vec![OpcodeInfo::new(0x8A, AddressingMode::Implied, 2, 0, ProcessorFlags::NZ)],
			"Transfer X to Accumulator",
		),
	);

	table.insert(
		Mnemonic::Tya,
		Instruction::new(
			Mnemonic::Tya,
			vec![OpcodeInfo::new(0x98, AddressingMode::Implied, 2, 0, ProcessorFlags::NZ)],
			"Transfer Y to Accumulator",
		),
	);

	// Stack Operations
	table.insert(
		Mnemonic::Tsx,
		Instruction::new(
			Mnemonic::Tsx,
			vec![OpcodeInfo::new(0xBA, AddressingMode::Implied, 2, 0, ProcessorFlags::NZ)],
			"Transfer Stack Pointer to X",
		),
	);

	table.insert(
		Mnemonic::Txs,
		Instruction::new(
			Mnemonic::Txs,
			vec![OpcodeInfo::new(0x9A, AddressingMode::Implied, 2, 0, ProcessorFlags::NONE)],
			"Transfer X to Stack Pointer",
		),
	);

	table.insert(
		Mnemonic::Pha,
		Instruction::new(
			Mnemonic::Pha,
			vec![OpcodeInfo::new(0x48, AddressingMode::Implied, 3, 0, ProcessorFlags::NONE)],
			"Push Accumulator on Stack",
		),
	);

	table.insert(
		Mnemonic::Php,
		Instruction::new(
			Mnemonic::Php,
			vec![OpcodeInfo::new(0x08, AddressingMode::Implied, 3, 0, ProcessorFlags::NONE)],
			"Push Processor Status on Stack",
		),
	);

	table.insert(
		Mnemonic::Pla,
		Instruction::new(
			Mnemonic::Pla,
			vec![OpcodeInfo::new(0x68, AddressingMode::Implied, 4, 0, ProcessorFlags::NZ)],
			"Pull Accumulator from Stack",
		),
	);

	table.insert(
		Mnemonic::Plp,
		Instruction::new(
			Mnemonic::Plp,
			vec![OpcodeInfo::new(0x28, AddressingMode::Implied, 4, 0, ProcessorFlags::NVZC)],
			"Pull Processor Status from Stack",
		),
	);

	// Logical Operations
	table.insert(
		Mnemonic::And,
		Instruction::new(
			Mnemonic::And,
			vec![
				OpcodeInfo::new(0x29, AddressingMode::Immediate, 2, 0, ProcessorFlags::NZ),
				OpcodeInfo::new(0x25, AddressingMode::ZeroPage, 3, 0, ProcessorFlags::NZ),
				OpcodeInfo::new(0x35, AddressingMode::ZeroPageX, 4, 0, ProcessorFlags::NZ),
				OpcodeInfo::new(0x2D, AddressingMode::Absolute, 4, 0, ProcessorFlags::NZ),
				OpcodeInfo::new(0x3D, AddressingMode::AbsoluteX, 4, 1, ProcessorFlags::NZ),
				OpcodeInfo::new(0x39, AddressingMode::AbsoluteY, 4, 1, ProcessorFlags::NZ),
				OpcodeInfo::new(0x21, AddressingMode::IndexedIndirect, 6, 0, ProcessorFlags::NZ),
				OpcodeInfo::new(0x31, AddressingMode::IndirectIndexed, 5, 1, ProcessorFlags::NZ),
			],
			"AND Memory with Accumulator",
		),
	);

	// System Instructions
	table.insert(
		Mnemonic::Nop,
		Instruction::new(
			Mnemonic::Nop,
			vec![OpcodeInfo::new(0xEA, AddressingMode::Implied, 2, 0, ProcessorFlags::NONE)],
			"No Operation",
		),
	);

	table.insert(
		Mnemonic::Brk,
		Instruction::new(
			Mnemonic::Brk,
			vec![OpcodeInfo::new(0x00, AddressingMode::Implied, 7, 0, ProcessorFlags::NONE)],
			"Force Break",
		),
	);

	table.insert(
		Mnemonic::Rti,
		Instruction::new(
			Mnemonic::Rti,
			vec![OpcodeInfo::new(0x40, AddressingMode::Implied, 6, 0, ProcessorFlags::NVZC)],
			"Return from Interrupt",
		),
	);

	// Branch Instructions
	table.insert(
		Mnemonic::Bcc,
		Instruction::new(
			Mnemonic::Bcc,
			vec![OpcodeInfo::new(0x90, AddressingMode::Relative, 2, 1, ProcessorFlags::NONE)],
			"Branch if Carry Clear",
		),
	);

	table.insert(
		Mnemonic::Bcs,
		Instruction::new(
			Mnemonic::Bcs,
			vec![OpcodeInfo::new(0xB0, AddressingMode::Relative, 2, 1, ProcessorFlags::NONE)],
			"Branch if Carry Set",
		),
	);

	table.insert(
		Mnemonic::Beq,
		Instruction::new(
			Mnemonic::Beq,
			vec![OpcodeInfo::new(0xF0, AddressingMode::Relative, 2, 1, ProcessorFlags::NONE)],
			"Branch if Equal",
		),
	);

	table.insert(
		Mnemonic::Bne,
		Instruction::new(
			Mnemonic::Bne,
			vec![OpcodeInfo::new(0xD0, AddressingMode::Relative, 2, 1, ProcessorFlags::NONE)],
			"Branch if Not Equal",
		),
	);

	// Status Flag Changes
	table.insert(
		Mnemonic::Clc,
		Instruction::new(
			Mnemonic::Clc,
			vec![OpcodeInfo::new(
				0x18,
				AddressingMode::Implied,
				2,
				0,
				ProcessorFlags {
					c: true,
					..ProcessorFlags::NONE
				},
			)],
			"Clear Carry Flag",
		),
	);

	table.insert(
		Mnemonic::Sec,
		Instruction::new(
			Mnemonic::Sec,
			vec![OpcodeInfo::new(
				0x38,
				AddressingMode::Implied,
				2,
				0,
				ProcessorFlags {
					c: true,
					..ProcessorFlags::NONE
				},
			)],
			"Set Carry Flag",
		),
	);

	table.insert(
		Mnemonic::Sei,
		Instruction::new(
			Mnemonic::Sei,
			vec![OpcodeInfo::new(
				0x78,
				AddressingMode::Implied,
				2,
				0,
				ProcessorFlags {
					i: true,
					..ProcessorFlags::NONE
				},
			)],
			"Set Interrupt Disable",
		),
	);

	table.insert(
		Mnemonic::Cli,
		Instruction::new(
			Mnemonic::Cli,
			vec![OpcodeInfo::new(
				0x58,
				AddressingMode::Implied,
				2,
				0,
				ProcessorFlags {
					i: true,
					..ProcessorFlags::NONE
				},
			)],
			"Clear Interrupt Disable",
		),
	);

	// Jump Instructions
	table.insert(
		Mnemonic::Jmp,
		Instruction::new(
			Mnemonic::Jmp,
			vec![
				OpcodeInfo::new(0x4C, AddressingMode::Absolute, 3, 0, ProcessorFlags::NONE),
				OpcodeInfo::new(0x6C, AddressingMode::Indirect, 5, 0, ProcessorFlags::NONE),
			],
			"Jump to New Location",
		),
	);

	table.insert(
		Mnemonic::Jsr,
		Instruction::new(
			Mnemonic::Jsr,
			vec![OpcodeInfo::new(0x20, AddressingMode::Absolute, 6, 0, ProcessorFlags::NONE)],
			"Jump to Subroutine",
		),
	);

	table.insert(
		Mnemonic::Rts,
		Instruction::new(
			Mnemonic::Rts,
			vec![OpcodeInfo::new(0x60, AddressingMode::Implied, 6, 0, ProcessorFlags::NONE)],
			"Return from Subroutine",
		),
	);

	// TODO: Add remaining instructions (ADC, SBC, CMP, INC, DEC, ASL, LSR, ROL, ROR, etc.)
	// This is a substantial but straightforward extension of the pattern above

	table
}

/// Get the global instruction table (lazy initialization)
pub fn instruction_table() -> &'static HashMap<Mnemonic, Instruction> {
	use std::sync::OnceLock;
	static INSTRUCTION_TABLE: OnceLock<HashMap<Mnemonic, Instruction>> = OnceLock::new();
	INSTRUCTION_TABLE.get_or_init(build_instruction_table)
}

/// Look up an instruction by mnemonic
pub fn get_instruction(mnemonic: Mnemonic) -> Option<&'static Instruction> {
	instruction_table().get(&mnemonic)
}

/// Look up an opcode by instruction and addressing mode
pub fn get_opcode(
	mnemonic: Mnemonic,
	addressing_mode: AddressingMode,
) -> Option<&'static OpcodeInfo> {
	get_instruction(mnemonic)?.get_opcode(addressing_mode)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_mnemonic_parsing() {
		assert_eq!(Mnemonic::parse("LDA"), Some(Mnemonic::Lda));
		assert_eq!(Mnemonic::parse("lda"), Some(Mnemonic::Lda));
		assert_eq!(Mnemonic::parse("LDX"), Some(Mnemonic::Ldx));
		assert_eq!(Mnemonic::parse("NOP"), Some(Mnemonic::Nop));
		assert_eq!(Mnemonic::parse("INVALID"), None);
	}

	#[test]
	fn test_mnemonic_categories() {
		assert_eq!(Mnemonic::Lda.category(), InstructionCategory::LoadStore);
		assert_eq!(Mnemonic::Tax.category(), InstructionCategory::Transfer);
		assert_eq!(Mnemonic::Pha.category(), InstructionCategory::Stack);
		assert_eq!(Mnemonic::And.category(), InstructionCategory::Logical);
		assert_eq!(Mnemonic::Adc.category(), InstructionCategory::Arithmetic);
		assert_eq!(Mnemonic::Bcc.category(), InstructionCategory::Branch);
		assert_eq!(Mnemonic::Nop.category(), InstructionCategory::System);
	}

	#[test]
	fn test_official_vs_unofficial() {
		assert!(Mnemonic::Lda.is_official());
		assert!(!Mnemonic::Lda.is_unofficial());
		assert!(Mnemonic::Lax.is_unofficial());
		assert!(!Mnemonic::Lax.is_official());
	}

	#[test]
	fn test_opcode_info() {
		let opcode = OpcodeInfo::new(0xA9, AddressingMode::Immediate, 2, 0, ProcessorFlags::NZ);
		assert_eq!(opcode.opcode, 0xA9);
		assert_eq!(opcode.addressing_mode, AddressingMode::Immediate);
		assert_eq!(opcode.cycles, 2);
		assert_eq!(opcode.size(), 2);
		assert_eq!(opcode.effective_cycles(false), 2);
		assert_eq!(opcode.effective_cycles(true), 2);
	}

	#[test]
	fn test_processor_flags() {
		assert!(!ProcessorFlags::NONE.n);
		assert!(!ProcessorFlags::NONE.z);
		assert!(ProcessorFlags::NZ.n);
		assert!(ProcessorFlags::NZ.z);
		assert!(!ProcessorFlags::NZ.c);
	}

	#[test]
	fn test_instruction_table() {
		let table = instruction_table();
		assert!(table.contains_key(&Mnemonic::Lda));
		assert!(table.contains_key(&Mnemonic::Nop));

		let lda = table.get(&Mnemonic::Lda).unwrap();
		assert_eq!(lda.mnemonic, Mnemonic::Lda);
		assert!(lda.supports_addressing_mode(AddressingMode::Immediate));
		assert!(lda.supports_addressing_mode(AddressingMode::ZeroPage));
		assert!(!lda.supports_addressing_mode(AddressingMode::Relative));
	}

	#[test]
	fn test_get_instruction() {
		let lda = get_instruction(Mnemonic::Lda).unwrap();
		assert_eq!(lda.mnemonic, Mnemonic::Lda);

		let opcode = lda.get_opcode(AddressingMode::Immediate).unwrap();
		assert_eq!(opcode.opcode, 0xA9);
		assert_eq!(opcode.cycles, 2);
	}

	#[test]
	fn test_get_opcode() {
		let opcode = get_opcode(Mnemonic::Lda, AddressingMode::Immediate).unwrap();
		assert_eq!(opcode.opcode, 0xA9);

		let opcode = get_opcode(Mnemonic::Nop, AddressingMode::Implied).unwrap();
		assert_eq!(opcode.opcode, 0xEA);

		// Invalid combination
		assert!(get_opcode(Mnemonic::Lda, AddressingMode::Relative).is_none());
	}

	#[test]
	fn test_addressing_modes() {
		let lda = get_instruction(Mnemonic::Lda).unwrap();
		let modes = lda.addressing_modes();
		assert!(modes.contains(&AddressingMode::Immediate));
		assert!(modes.contains(&AddressingMode::ZeroPage));
		assert!(modes.contains(&AddressingMode::Absolute));
		assert!(!modes.contains(&AddressingMode::Relative));
	}
}
