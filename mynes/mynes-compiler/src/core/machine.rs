//! Machine description and configuration
//!
//! This module defines the target machine characteristics and provides
//! machine-specific functionality for the NES compiler.

use crate::error::AssemblyResult;
use crate::symbols::SymbolTable;

/// Machine state during assembly
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineState {
	/// Initial state
	Initial,
	/// Processing source files
	Processing,
	/// Generating output
	Generating,
	/// Assembly complete
	Complete,
	/// Assembly failed
	Failed,
}

impl Default for MachineState {
	fn default() -> Self {
		Self::Initial
	}
}

impl std::fmt::Display for MachineState {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Initial => write!(f, "Initial"),
			Self::Processing => write!(f, "Processing"),
			Self::Generating => write!(f, "Generating"),
			Self::Complete => write!(f, "Complete"),
			Self::Failed => write!(f, "Failed"),
		}
	}
}

/// Supported target machine types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineType {
	/// Nintendo Entertainment System / Famicom
	Nes,
}

impl Default for MachineType {
	fn default() -> Self {
		Self::Nes
	}
}

impl std::fmt::Display for MachineType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			MachineType::Nes => write!(f, "NES"),
		}
	}
}

/// Machine description containing target-specific information
#[derive(Debug, Clone)]
pub struct Machine {
	/// Machine type
	pub machine_type: MachineType,
	/// Machine name for display
	pub name: String,
	/// ROM file extension
	pub rom_extension: String,
	/// Environment variable for include paths
	pub include_env: String,
	/// Zero page memory limit
	pub zero_page_limit: u16,
	/// RAM memory limit
	pub ram_limit: u16,
	/// RAM base address
	pub ram_base: u16,
	/// RAM page number
	pub ram_page: u8,
	/// Default RAM bank
	pub ram_bank: usize,
	/// CPU architecture
	pub cpu: CpuType,
	/// Memory layout configuration
	pub memory_layout: MemoryLayout,
}

/// CPU architecture types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuType {
	/// MOS Technology 6502
	Cpu6502,
	/// Ricoh 2A03 (NES variant of 6502)
	Ricoh2A03,
}

/// Memory layout configuration
#[derive(Debug, Clone)]
pub struct MemoryLayout {
	/// Zero page range
	pub zero_page: AddressRange,
	/// Stack range
	pub stack: AddressRange,
	/// RAM range
	pub ram: AddressRange,
	/// PRG ROM range
	pub prg_rom: AddressRange,
	/// CHR ROM range (if applicable)
	pub chr_rom: Option<AddressRange>,
	/// I/O registers range
	pub io_registers: AddressRange,
}

/// Address range specification
#[derive(Debug, Clone, Copy)]
pub struct AddressRange {
	pub start: u16,
	pub end: u16,
}

impl AddressRange {
	/// Create a new address range
	pub fn new(start: u16, end: u16) -> Self {
		Self {
			start,
			end,
		}
	}

	/// Check if an address is within this range
	pub fn contains(&self, address: u16) -> bool {
		address >= self.start && address <= self.end
	}

	/// Get the size of this range
	pub fn size(&self) -> u16 {
		self.end.wrapping_sub(self.start).wrapping_add(1)
	}
}

impl Machine {
	/// Create a new machine description for the specified type
	pub fn new(machine_type: MachineType) -> Self {
		match machine_type {
			MachineType::Nes => Self::new_nes(),
		}
	}

	/// Create NES machine configuration
	fn new_nes() -> Self {
		Self {
			machine_type: MachineType::Nes,
			name: "Nintendo Entertainment System".to_string(),
			rom_extension: "nes".to_string(),
			include_env: "NES_INCLUDE".to_string(),
			zero_page_limit: 0x00FF,
			ram_limit: 0x07FF,
			ram_base: 0x0000,
			ram_page: 0,
			ram_bank: 0,
			cpu: CpuType::Ricoh2A03,
			memory_layout: MemoryLayout {
				zero_page: AddressRange::new(0x0000, 0x00FF),
				stack: AddressRange::new(0x0100, 0x01FF),
				ram: AddressRange::new(0x0200, 0x07FF),
				prg_rom: AddressRange::new(0x8000, 0xFFFF),
				chr_rom: Some(AddressRange::new(0x0000, 0x1FFF)),
				io_registers: AddressRange::new(0x2000, 0x401F),
			},
		}
	}

	/// Add machine-specific predefined symbols to the symbol table
	pub fn add_predefined_symbols(&self, symbols: &mut SymbolTable) -> AssemblyResult<()> {
		match self.machine_type {
			MachineType::Nes => self.add_nes_symbols(symbols),
		}
	}

	/// Add NES-specific predefined symbols
	fn add_nes_symbols(&self, symbols: &mut SymbolTable) -> AssemblyResult<()> {
		// PPU registers
		symbols.define_constant("PPUCTRL".to_string(), 0x2000)?;
		symbols.define_constant("PPU_CTRL".to_string(), 0x2000)?;
		symbols.define_constant("PPUMASK".to_string(), 0x2001)?;
		symbols.define_constant("PPU_MASK".to_string(), 0x2001)?;
		symbols.define_constant("PPUSTATUS".to_string(), 0x2002)?;
		symbols.define_constant("PPU_STATUS".to_string(), 0x2002)?;
		symbols.define_constant("PPUSTAT".to_string(), 0x2002)?;
		symbols.define_constant("OAMADDR".to_string(), 0x2003)?;
		symbols.define_constant("OAM_ADDR".to_string(), 0x2003)?;
		symbols.define_constant("PPU_OAM_ADDR".to_string(), 0x2003)?;
		symbols.define_constant("OAMDATA".to_string(), 0x2004)?;
		symbols.define_constant("OAM_DATA".to_string(), 0x2004)?;
		symbols.define_constant("PPU_OAM_DATA".to_string(), 0x2004)?;
		symbols.define_constant("PPUSCROLL".to_string(), 0x2005)?;
		symbols.define_constant("PPU_SCROLL".to_string(), 0x2005)?;
		symbols.define_constant("PPUADDR".to_string(), 0x2006)?;
		symbols.define_constant("PPU_ADDR".to_string(), 0x2006)?;
		symbols.define_constant("PPUDATA".to_string(), 0x2007)?;
		symbols.define_constant("PPU_DATA".to_string(), 0x2007)?;

		// APU registers - Square wave 1
		symbols.define_constant("SQ1VOL".to_string(), 0x4000)?;
		symbols.define_constant("SQ1_VOL".to_string(), 0x4000)?;
		symbols.define_constant("SQ1SWEEP".to_string(), 0x4001)?;
		symbols.define_constant("SQ1_SWEEP".to_string(), 0x4001)?;
		symbols.define_constant("SQ1LO".to_string(), 0x4002)?;
		symbols.define_constant("SQ1_LO".to_string(), 0x4002)?;
		symbols.define_constant("SQ1HI".to_string(), 0x4003)?;
		symbols.define_constant("SQ1_HI".to_string(), 0x4003)?;

		// APU registers - Square wave 2
		symbols.define_constant("SQ2VOL".to_string(), 0x4004)?;
		symbols.define_constant("SQ2_VOL".to_string(), 0x4004)?;
		symbols.define_constant("SQ2SWEEP".to_string(), 0x4005)?;
		symbols.define_constant("SQ2_SWEEP".to_string(), 0x4005)?;
		symbols.define_constant("SQ2LO".to_string(), 0x4006)?;
		symbols.define_constant("SQ2_LO".to_string(), 0x4006)?;
		symbols.define_constant("SQ2HI".to_string(), 0x4007)?;
		symbols.define_constant("SQ2_HI".to_string(), 0x4007)?;

		// APU registers - Triangle wave
		symbols.define_constant("TRILINEAR".to_string(), 0x4008)?;
		symbols.define_constant("TRI_LINEAR".to_string(), 0x4008)?;
		symbols.define_constant("TRILO".to_string(), 0x400A)?;
		symbols.define_constant("TRI_LO".to_string(), 0x400A)?;
		symbols.define_constant("TRIHI".to_string(), 0x400B)?;
		symbols.define_constant("TRI_HI".to_string(), 0x400B)?;

		// APU registers - Noise channel
		symbols.define_constant("NOISEVOL".to_string(), 0x400C)?;
		symbols.define_constant("NOISE_VOL".to_string(), 0x400C)?;
		symbols.define_constant("NOISELO".to_string(), 0x400E)?;
		symbols.define_constant("NOISE_LO".to_string(), 0x400E)?;
		symbols.define_constant("NOISEHI".to_string(), 0x400F)?;
		symbols.define_constant("NOISE_HI".to_string(), 0x400F)?;

		// APU registers - DMC channel
		symbols.define_constant("DMCFREQ".to_string(), 0x4010)?;
		symbols.define_constant("DMC_FREQ".to_string(), 0x4010)?;
		symbols.define_constant("DMCRAW".to_string(), 0x4011)?;
		symbols.define_constant("DMC_RAW".to_string(), 0x4011)?;
		symbols.define_constant("DMCSTART".to_string(), 0x4012)?;
		symbols.define_constant("DMC_START".to_string(), 0x4012)?;
		symbols.define_constant("DMCLEN".to_string(), 0x4013)?;
		symbols.define_constant("DMC_LEN".to_string(), 0x4013)?;

		// Other I/O registers
		symbols.define_constant("OAMDMA".to_string(), 0x4014)?;
		symbols.define_constant("OAM_DMA".to_string(), 0x4014)?;
		symbols.define_constant("PPU_OAM_DMA".to_string(), 0x4014)?;
		symbols.define_constant("APUSTATUS".to_string(), 0x4015)?;
		symbols.define_constant("APU_STATUS".to_string(), 0x4015)?;

		// Controller registers
		symbols.define_constant("JOY1".to_string(), 0x4016)?;
		symbols.define_constant("JOY2".to_string(), 0x4017)?;
		symbols.define_constant("JOY2FRAME".to_string(), 0x4017)?;
		symbols.define_constant("JOY2_FRAME".to_string(), 0x4017)?;

		// Hardware vectors
		symbols.define_constant("NMI_VECTOR".to_string(), 0xFFFA)?;
		symbols.define_constant("RESET_VECTOR".to_string(), 0xFFFC)?;
		symbols.define_constant("IRQ_VECTOR".to_string(), 0xFFFE)?;

		// Common memory boundaries
		symbols.define_constant("RAM_START".to_string(), 0x0000)?;
		symbols.define_constant("RAM_END".to_string(), 0x07FF)?;
		symbols.define_constant("PPU_START".to_string(), 0x2000)?;
		symbols.define_constant("PPU_END".to_string(), 0x3FFF)?;
		symbols.define_constant("APU_START".to_string(), 0x4000)?;
		symbols.define_constant("APU_END".to_string(), 0x401F)?;
		symbols.define_constant("PRG_START".to_string(), 0x8000)?;
		symbols.define_constant("PRG_END".to_string(), 0xFFFF)?;

		Ok(())
	}

	/// Check if an address is valid for the current machine
	pub fn is_valid_address(&self, _address: u16) -> bool {
		match self.machine_type {
			MachineType::Nes => {
				// All 16-bit addresses are valid on NES
				true
			}
		}
	}

	/// Get the page number for an address
	pub fn address_to_page(&self, address: u16) -> u8 {
		(address >> 13) as u8
	}

	/// Check if an address is in zero page
	pub fn is_zero_page(&self, address: u16) -> bool {
		self.memory_layout.zero_page.contains(address)
	}

	/// Check if an address is in RAM
	pub fn is_ram(&self, address: u16) -> bool {
		self.memory_layout.zero_page.contains(address)
			|| self.memory_layout.stack.contains(address)
			|| self.memory_layout.ram.contains(address)
	}

	/// Check if an address is in ROM
	pub fn is_rom(&self, address: u16) -> bool {
		self.memory_layout.prg_rom.contains(address)
	}

	/// Check if an address is an I/O register
	pub fn is_io_register(&self, address: u16) -> bool {
		self.memory_layout.io_registers.contains(address)
	}

	/// Get the default bank size for this machine
	pub fn bank_size(&self) -> usize {
		match self.machine_type {
			MachineType::Nes => 8192, // 8KB banks
		}
	}

	/// Get the maximum number of banks supported
	pub fn max_banks(&self) -> usize {
		match self.machine_type {
			MachineType::Nes => 4096,
		}
	}

	/// Get the CPU type for this machine
	pub fn cpu_type(&self) -> CpuType {
		self.cpu
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_address_range() {
		let range = AddressRange::new(0x2000, 0x3FFF);
		assert!(range.contains(0x2000));
		assert!(range.contains(0x2500));
		assert!(range.contains(0x3FFF));
		assert!(!range.contains(0x1FFF));
		assert!(!range.contains(0x4000));
		assert_eq!(range.size(), 0x2000);
	}

	#[test]
	fn test_nes_machine_creation() {
		let machine = Machine::new(MachineType::Nes);
		assert_eq!(machine.machine_type, MachineType::Nes);
		assert_eq!(machine.name, "Nintendo Entertainment System");
		assert_eq!(machine.rom_extension, "nes");
		assert_eq!(machine.cpu, CpuType::Ricoh2A03);
		assert_eq!(machine.bank_size(), 8192);
		assert_eq!(machine.max_banks(), 4096);
	}

	#[test]
	fn test_nes_memory_layout() {
		let machine = Machine::new(MachineType::Nes);

		// Test zero page
		assert!(machine.is_zero_page(0x00));
		assert!(machine.is_zero_page(0xFF));
		assert!(!machine.is_zero_page(0x100));

		// Test RAM
		assert!(machine.is_ram(0x00)); // Zero page is RAM
		assert!(machine.is_ram(0x200)); // Regular RAM
		assert!(!machine.is_ram(0x2000)); // PPU registers

		// Test ROM
		assert!(machine.is_rom(0x8000));
		assert!(machine.is_rom(0xFFFF));
		assert!(!machine.is_rom(0x7FFF));

		// Test I/O registers
		assert!(machine.is_io_register(0x2000)); // PPU
		assert!(machine.is_io_register(0x4000)); // APU
		assert!(!machine.is_io_register(0x1FFF));
	}

	#[test]
	fn test_address_validation() {
		let machine = Machine::new(MachineType::Nes);
		assert!(machine.is_valid_address(0x0000));
		assert!(machine.is_valid_address(0x8000));
		assert!(machine.is_valid_address(0xFFFF));
	}

	#[test]
	fn test_page_calculation() {
		let machine = Machine::new(MachineType::Nes);
		assert_eq!(machine.address_to_page(0x0000), 0);
		assert_eq!(machine.address_to_page(0x1FFF), 0);
		assert_eq!(machine.address_to_page(0x2000), 1);
		assert_eq!(machine.address_to_page(0x8000), 4);
		assert_eq!(machine.address_to_page(0xFFFF), 7);
	}

	#[test]
	fn test_machine_type_display() {
		assert_eq!(MachineType::Nes.to_string(), "NES");
	}
}
