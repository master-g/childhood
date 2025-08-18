//! NES platform implementation for the compiler
//!
//! This module provides NES-specific functionality including memory layout,
//! iNES header generation, and NES hardware register definitions.

use crate::error::{AssemblyError, AssemblyResult};
use crate::platform::Platform;

/// NES platform configuration
#[derive(Debug, Clone)]
pub struct NesConfig {
	/// Number of PRG ROM banks (16KB each)
	pub prg_banks: u8,
	/// Number of CHR ROM banks (8KB each)
	pub chr_banks: u8,
	/// Mapper number
	pub mapper: u8,
	/// Submapper number
	pub submapper: u8,
	/// Mirroring type
	pub mirroring: Mirroring,
	/// Whether the cartridge has battery-backed RAM
	pub battery: bool,
	/// Whether the cartridge has a trainer
	pub trainer: bool,
	/// Console timing type
	pub timing: Timing,
}

/// NES mirroring types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mirroring {
	/// Horizontal mirroring
	Horizontal = 0,
	/// Vertical mirroring
	Vertical = 1,
	/// Single screen mirroring
	SingleScreen = 2,
	/// Four screen mirroring
	FourScreen = 3,
}

/// NES timing types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Timing {
	/// NTSC timing
	Ntsc = 0,
	/// PAL timing
	Pal = 1,
	/// Multiple/dual compatible
	Multiple = 2,
	/// Dendy timing
	Dendy = 3,
}

impl Default for NesConfig {
	fn default() -> Self {
		Self {
			prg_banks: 1,
			chr_banks: 1,
			mapper: 0,
			submapper: 0,
			mirroring: Mirroring::Horizontal,
			battery: false,
			trainer: false,
			timing: Timing::Ntsc,
		}
	}
}

/// NES platform implementation
#[derive(Debug)]
pub struct NesPlatform {
	/// Platform configuration
	config: NesConfig,
}

impl NesPlatform {
	/// Create a new NES platform with default configuration
	pub fn new() -> Self {
		Self {
			config: NesConfig::default(),
		}
	}

	/// Create a new NES platform with custom configuration
	pub fn with_config(config: NesConfig) -> Self {
		Self {
			config,
		}
	}

	/// Get the current configuration
	pub fn config(&self) -> &NesConfig {
		&self.config
	}

	/// Set the configuration
	pub fn set_config(&mut self, config: NesConfig) {
		self.config = config;
	}

	/// Generate iNES 2.0 header
	pub fn generate_ines_header(&self) -> Vec<u8> {
		let mut header = vec![0u8; 16];

		// Magic signature "NES\x1A"
		header[0] = b'N';
		header[1] = b'E';
		header[2] = b'S';
		header[3] = 0x1A;

		// PRG ROM size (16KB units)
		header[4] = self.config.prg_banks;

		// CHR ROM size (8KB units)
		header[5] = self.config.chr_banks;

		// Flags 6
		let mut flags6 = 0u8;
		flags6 |= (self.config.mirroring as u8) & 0x01;
		if self.config.battery {
			flags6 |= 0x02;
		}
		if self.config.trainer {
			flags6 |= 0x04;
		}
		flags6 |= (self.config.mapper & 0x0F) << 4;
		header[6] = flags6;

		// Flags 7
		let mut flags7 = 0u8;
		flags7 |= (self.config.mapper & 0xF0);
		flags7 |= 0x08; // NES 2.0 identifier
		header[7] = flags7;

		// Mapper MSB and submapper
		header[8] = ((self.config.submapper & 0x0F) << 4);

		// PRG/CHR ROM size MSB
		header[9] = 0; // For now, assume sizes fit in LSB

		// PRG/CHR RAM size
		header[10] = 0;
		header[11] = 0;

		// Timing
		header[12] = self.config.timing as u8;

		// System type, misc ROMs, default expansion device
		header[13] = 0;
		header[14] = 0;
		header[15] = 0;

		header
	}
}

impl Platform for NesPlatform {
	fn name(&self) -> &str {
		"NES"
	}

	fn rom_extension(&self) -> &str {
		"nes"
	}

	fn bank_size(&self) -> usize {
		8192 // 8KB
	}

	fn max_banks(&self) -> usize {
		4096
	}

	fn validate_address(&self, address: u16) -> bool {
		// All 16-bit addresses are potentially valid on NES
		// The actual validity depends on the mapper and memory layout
		match address {
			// Zero page
			0x0000..=0x00FF => true,
			// Stack
			0x0100..=0x01FF => true,
			// RAM
			0x0200..=0x07FF => true,
			// PPU registers (mirrored)
			0x2000..=0x3FFF => true,
			// APU and I/O registers
			0x4000..=0x401F => true,
			// Cartridge space
			0x4020..=0xFFFF => true,
			// Everything else is invalid
			_ => false,
		}
	}

	fn generate_header(&self, prg_banks: u8, chr_banks: u8) -> AssemblyResult<Vec<u8>> {
		let mut config = self.config.clone();
		config.prg_banks = prg_banks;
		config.chr_banks = chr_banks;

		let platform = NesPlatform::with_config(config);
		Ok(platform.generate_ines_header())
	}

	fn predefined_symbols(&self) -> Vec<(String, i32)> {
		vec![
			// PPU registers
			("PPUCTRL".to_string(), 0x2000),
			("PPU_CTRL".to_string(), 0x2000),
			("PPUMASK".to_string(), 0x2001),
			("PPU_MASK".to_string(), 0x2001),
			("PPUSTATUS".to_string(), 0x2002),
			("PPUSTAT".to_string(), 0x2002),
			("PPU_STATUS".to_string(), 0x2002),
			("OAMADDR".to_string(), 0x2003),
			("OAM_ADDR".to_string(), 0x2003),
			("PPU_OAM_ADDR".to_string(), 0x2003),
			("OAMDATA".to_string(), 0x2004),
			("OAM_DATA".to_string(), 0x2004),
			("PPU_OAM_DATA".to_string(), 0x2004),
			("PPUSCROLL".to_string(), 0x2005),
			("PPU_SCROLL".to_string(), 0x2005),
			("PPUADDR".to_string(), 0x2006),
			("PPU_ADDR".to_string(), 0x2006),
			("PPUDATA".to_string(), 0x2007),
			("PPU_DATA".to_string(), 0x2007),
			// APU registers - Square 1
			("SQ1VOL".to_string(), 0x4000),
			("SQ1_VOL".to_string(), 0x4000),
			("SQ1SWEEP".to_string(), 0x4001),
			("SQ1_SWEEP".to_string(), 0x4001),
			("SQ1LO".to_string(), 0x4002),
			("SQ1_LO".to_string(), 0x4002),
			("SQ1HI".to_string(), 0x4003),
			("SQ1_HI".to_string(), 0x4003),
			// APU registers - Square 2
			("SQ2VOL".to_string(), 0x4004),
			("SQ2_VOL".to_string(), 0x4004),
			("SQ2SWEEP".to_string(), 0x4005),
			("SQ2_SWEEP".to_string(), 0x4005),
			("SQ2LO".to_string(), 0x4006),
			("SQ2_LO".to_string(), 0x4006),
			("SQ2HI".to_string(), 0x4007),
			("SQ2_HI".to_string(), 0x4007),
			// APU registers - Triangle
			("TRILINEAR".to_string(), 0x4008),
			("TRI_LINEAR".to_string(), 0x4008),
			("TRILO".to_string(), 0x400A),
			("TRI_LO".to_string(), 0x400A),
			("TRIHI".to_string(), 0x400B),
			("TRI_HI".to_string(), 0x400B),
			// APU registers - Noise
			("NOISEVOL".to_string(), 0x400C),
			("NOISE_VOL".to_string(), 0x400C),
			("NOISELO".to_string(), 0x400E),
			("NOISE_LO".to_string(), 0x400E),
			("NOISEHI".to_string(), 0x400F),
			("NOISE_HI".to_string(), 0x400F),
			// APU registers - DMC
			("DMCFREQ".to_string(), 0x4010),
			("DMC_FREQ".to_string(), 0x4010),
			("DMCRAW".to_string(), 0x4011),
			("DMC_RAW".to_string(), 0x4011),
			("DMCSTART".to_string(), 0x4012),
			("DMC_START".to_string(), 0x4012),
			("DMCLEN".to_string(), 0x4013),
			("DMC_LEN".to_string(), 0x4013),
			// Other I/O
			("OAMDMA".to_string(), 0x4014),
			("OAM_DMA".to_string(), 0x4014),
			("PPU_OAM_DMA".to_string(), 0x4014),
			("APUSTATUS".to_string(), 0x4015),
			("APU_STATUS".to_string(), 0x4015),
			("JOY1".to_string(), 0x4016),
			("JOY2".to_string(), 0x4017),
			("JOY2FRAME".to_string(), 0x4017),
			("JOY2_FRAME".to_string(), 0x4017),
			// Hardware vectors
			("NMI_VECTOR".to_string(), 0xFFFA),
			("RESET_VECTOR".to_string(), 0xFFFC),
			("IRQ_VECTOR".to_string(), 0xFFFE),
			// Memory boundaries
			("RAM_START".to_string(), 0x0000),
			("RAM_END".to_string(), 0x07FF),
			("PPU_START".to_string(), 0x2000),
			("PPU_END".to_string(), 0x3FFF),
			("APU_START".to_string(), 0x4000),
			("APU_END".to_string(), 0x401F),
			("PRG_START".to_string(), 0x8000),
			("PRG_END".to_string(), 0xFFFF),
		]
	}
}

impl Default for NesPlatform {
	fn default() -> Self {
		Self::new()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_nes_platform_creation() {
		let platform = NesPlatform::new();
		assert_eq!(platform.name(), "NES");
		assert_eq!(platform.rom_extension(), "nes");
		assert_eq!(platform.bank_size(), 8192);
	}

	#[test]
	fn test_address_validation() {
		let platform = NesPlatform::new();

		// Valid addresses
		assert!(platform.validate_address(0x0000)); // Zero page
		assert!(platform.validate_address(0x0200)); // RAM
		assert!(platform.validate_address(0x2000)); // PPU
		assert!(platform.validate_address(0x4000)); // APU
		assert!(platform.validate_address(0x8000)); // PRG ROM
		assert!(platform.validate_address(0xFFFF)); // Top of memory
	}

	#[test]
	fn test_ines_header_generation() {
		let platform = NesPlatform::new();
		let header = platform.generate_ines_header();

		assert_eq!(header.len(), 16);
		assert_eq!(&header[0..4], b"NES\x1A");
		assert_eq!(header[4], 1); // 1 PRG bank
		assert_eq!(header[5], 1); // 1 CHR bank
	}

	#[test]
	fn test_predefined_symbols() {
		let platform = NesPlatform::new();
		let symbols = platform.predefined_symbols();

		assert!(!symbols.is_empty());

		// Check for some key symbols
		assert!(symbols.iter().any(|(name, value)| name == "PPUCTRL" && *value == 0x2000));
		assert!(symbols.iter().any(|(name, value)| name == "RESET_VECTOR" && *value == 0xFFFC));
	}

	#[test]
	fn test_config_modification() {
		let mut platform = NesPlatform::new();

		let mut config = NesConfig::default();
		config.prg_banks = 2;
		config.chr_banks = 2;
		config.mapper = 1;

		platform.set_config(config);

		let header = platform.generate_ines_header();
		assert_eq!(header[4], 2); // 2 PRG banks
		assert_eq!(header[5], 2); // 2 CHR banks
		assert_eq!(header[6] >> 4, 1); // Mapper 1 (lower nibble)
	}

	#[test]
	fn test_mirroring_types() {
		assert_eq!(Mirroring::Horizontal as u8, 0);
		assert_eq!(Mirroring::Vertical as u8, 1);
		assert_eq!(Mirroring::SingleScreen as u8, 2);
		assert_eq!(Mirroring::FourScreen as u8, 3);
	}
}
