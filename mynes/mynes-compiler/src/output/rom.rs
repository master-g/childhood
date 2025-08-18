//! ROM generation for the NES assembler.
//!
//! This module provides functionality to generate NES ROM files in various formats,
//! primarily the iNES format (.nes files) used by most NES emulators.

use crate::config::{INesConfig, MirroringType, VideoSystem};
use crate::core::memory::{MemoryManager, SectionType};
use crate::error::{AssemblyError, AssemblyResult, SourcePos};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Supported ROM output formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RomFormat {
	/// iNES format (.nes)
	INes,
	/// Raw binary (no header)
	Binary,
	/// NES 2.0 format (extended iNES)
	Nes20,
}

impl Default for RomFormat {
	fn default() -> Self {
		Self::INes
	}
}

/// iNES header structure (16 bytes)
#[derive(Debug, Clone)]
pub struct INesHeader {
	/// Magic number "NES\x1A"
	pub magic: [u8; 4],
	/// Number of 16KB PRG ROM banks
	pub prg_banks: u8,
	/// Number of 8KB CHR ROM banks
	pub chr_banks: u8,
	/// Flags 6: mapper low nibble, mirroring, battery, trainer
	pub flags6: u8,
	/// Flags 7: mapper high nibble, NES 2.0 signature, PlayChoice-10, VS System
	pub flags7: u8,
	/// Flags 8: PRG RAM size (rarely used)
	pub flags8: u8,
	/// Flags 9: TV system flags (rarely used)
	pub flags9: u8,
	/// Flags 10: TV system, PRG RAM presence (unofficial, rarely used)
	pub flags10: u8,
	/// Unused padding bytes (should be zero)
	pub unused: [u8; 5],
}

impl INesHeader {
	/// Create a new iNES header with default values
	pub fn new() -> Self {
		Self {
			magic: [b'N', b'E', b'S', 0x1A],
			prg_banks: 1,
			chr_banks: 0,
			flags6: 0,
			flags7: 0,
			flags8: 0,
			flags9: 0,
			flags10: 0,
			unused: [0; 5],
		}
	}

	/// Create header from iNES configuration
	pub fn from_config(config: &INesConfig) -> Self {
		let mut header = Self::new();

		header.prg_banks = config.prg_banks;
		header.chr_banks = config.chr_banks;

		// Set flags6
		header.flags6 = (config.mapper & 0x0F) << 4; // Mapper low nibble

		// Mirroring
		match config.mirroring {
			MirroringType::Horizontal => {
				// Bit 0 = 0 for horizontal mirroring
			}
			MirroringType::Vertical => {
				header.flags6 |= 0x01; // Bit 0 = 1 for vertical mirroring
			}
			MirroringType::FourScreen => {
				header.flags6 |= 0x08; // Bit 3 = 1 for four-screen
			}
			MirroringType::SingleScreen => {
				// Use horizontal mirroring as default for single screen
			}
		}

		// Battery-backed PRG RAM
		if config.battery {
			header.flags6 |= 0x02; // Bit 1 = 1 for battery
		}

		// Trainer
		if config.trainer {
			header.flags6 |= 0x04; // Bit 2 = 1 for trainer
		}

		// Set flags7
		header.flags7 = (config.mapper & 0xF0); // Mapper high nibble

		// Video system
		match config.video_system {
			VideoSystem::Ntsc => {
				// Bit 0 = 0 for NTSC (default)
			}
			VideoSystem::Pal => {
				header.flags9 |= 0x01; // Use flags9 bit 0 for PAL
			}
			VideoSystem::Dual => {
				// Leave as NTSC default
			}
		}

		header
	}

	/// Convert header to bytes
	pub fn to_bytes(&self) -> [u8; 16] {
		[
			self.magic[0],
			self.magic[1],
			self.magic[2],
			self.magic[3],
			self.prg_banks,
			self.chr_banks,
			self.flags6,
			self.flags7,
			self.flags8,
			self.flags9,
			self.flags10,
			self.unused[0],
			self.unused[1],
			self.unused[2],
			self.unused[3],
			self.unused[4],
		]
	}

	/// Validate the header
	pub fn validate(&self) -> AssemblyResult<()> {
		// Check magic number
		if self.magic != [b'N', b'E', b'S', 0x1A] {
			return Err(AssemblyError::config("Invalid iNES magic number".to_string()));
		}

		// Check PRG ROM size
		if self.prg_banks == 0 {
			return Err(AssemblyError::config("PRG ROM size cannot be zero".to_string()));
		}

		// Check for reasonable limits
		if self.prg_banks > 32 {
			return Err(AssemblyError::config(format!(
				"PRG ROM size too large: {} banks (max 32)",
				self.prg_banks
			)));
		}

		if self.chr_banks > 32 {
			return Err(AssemblyError::config(format!(
				"CHR ROM size too large: {} banks (max 32)",
				self.chr_banks
			)));
		}

		Ok(())
	}

	/// Get total PRG ROM size in bytes
	pub fn prg_size(&self) -> usize {
		self.prg_banks as usize * 16384 // 16KB per bank
	}

	/// Get total CHR ROM size in bytes
	pub fn chr_size(&self) -> usize {
		self.chr_banks as usize * 8192 // 8KB per bank
	}

	/// Get mapper number
	pub fn mapper(&self) -> u8 {
		(self.flags6 >> 4) | (self.flags7 & 0xF0)
	}

	/// Check if has trainer
	pub fn has_trainer(&self) -> bool {
		(self.flags6 & 0x04) != 0
	}

	/// Check if has battery
	pub fn has_battery(&self) -> bool {
		(self.flags6 & 0x02) != 0
	}

	/// Get mirroring type
	pub fn mirroring(&self) -> MirroringType {
		if (self.flags6 & 0x08) != 0 {
			MirroringType::FourScreen
		} else if (self.flags6 & 0x01) != 0 {
			MirroringType::Vertical
		} else {
			MirroringType::Horizontal
		}
	}
}

impl Default for INesHeader {
	fn default() -> Self {
		Self::new()
	}
}

/// ROM data container
#[derive(Debug, Clone)]
pub struct RomData {
	/// PRG ROM data (program code)
	pub prg_rom: Vec<u8>,
	/// CHR ROM data (character/graphics data)
	pub chr_rom: Vec<u8>,
	/// Optional trainer data (512 bytes)
	pub trainer: Option<Vec<u8>>,
}

impl RomData {
	/// Create new empty ROM data
	pub fn new() -> Self {
		Self {
			prg_rom: Vec::new(),
			chr_rom: Vec::new(),
			trainer: None,
		}
	}

	/// Create ROM data with specified sizes
	pub fn with_sizes(prg_size: usize, chr_size: usize) -> Self {
		Self {
			prg_rom: vec![0; prg_size],
			chr_rom: vec![0; chr_size],
			trainer: None,
		}
	}

	/// Add trainer data
	pub fn with_trainer(mut self, trainer_data: Vec<u8>) -> AssemblyResult<Self> {
		if trainer_data.len() != 512 {
			return Err(AssemblyError::config(format!(
				"Trainer must be exactly 512 bytes, got {}",
				trainer_data.len()
			)));
		}
		self.trainer = Some(trainer_data);
		Ok(self)
	}

	/// Set PRG ROM data
	pub fn set_prg_rom(&mut self, data: Vec<u8>) {
		self.prg_rom = data;
	}

	/// Set CHR ROM data
	pub fn set_chr_rom(&mut self, data: Vec<u8>) {
		self.chr_rom = data;
	}

	/// Get total size in bytes (excluding header)
	pub fn total_size(&self) -> usize {
		let trainer_size = if self.trainer.is_some() {
			512
		} else {
			0
		};
		trainer_size + self.prg_rom.len() + self.chr_rom.len()
	}

	/// Validate ROM data against header
	pub fn validate(&self, header: &INesHeader) -> AssemblyResult<()> {
		// Check PRG ROM size
		let expected_prg_size = header.prg_size();
		if self.prg_rom.len() != expected_prg_size {
			return Err(AssemblyError::config(format!(
				"PRG ROM size mismatch: expected {} bytes, got {}",
				expected_prg_size,
				self.prg_rom.len()
			)));
		}

		// Check CHR ROM size
		let expected_chr_size = header.chr_size();
		if self.chr_rom.len() != expected_chr_size {
			return Err(AssemblyError::config(format!(
				"CHR ROM size mismatch: expected {} bytes, got {}",
				expected_chr_size,
				self.chr_rom.len()
			)));
		}

		// Check trainer
		if header.has_trainer() && self.trainer.is_none() {
			return Err(AssemblyError::config(
				"Header indicates trainer but no trainer data provided".to_string(),
			));
		}

		if !header.has_trainer() && self.trainer.is_some() {
			return Err(AssemblyError::config(
				"Trainer data provided but header doesn't indicate trainer".to_string(),
			));
		}

		Ok(())
	}
}

impl Default for RomData {
	fn default() -> Self {
		Self::new()
	}
}

/// ROM generator for creating NES ROM files
#[derive(Debug)]
pub struct RomGenerator {
	/// Output format
	format: RomFormat,
	/// Fill unused space with zeros
	zero_fill: bool,
}

impl RomGenerator {
	/// Create a new ROM generator
	pub fn new() -> Self {
		Self {
			format: RomFormat::default(),
			zero_fill: false,
		}
	}

	/// Set output format
	pub fn with_format(mut self, format: RomFormat) -> Self {
		self.format = format;
		self
	}

	/// Enable zero filling
	pub fn with_zero_fill(mut self, zero_fill: bool) -> Self {
		self.zero_fill = zero_fill;
		self
	}

	/// Generate ROM file from memory manager
	pub fn generate_from_memory(
		&self,
		memory: &MemoryManager,
		config: &INesConfig,
		output_path: &Path,
	) -> AssemblyResult<Vec<u8>> {
		// Extract ROM data from memory manager
		let rom_data = self.extract_rom_data(memory, config)?;

		// Create header
		let header = INesHeader::from_config(config);

		// Generate and write ROM
		self.generate_rom(&header, &rom_data, output_path)
	}

	/// Generate ROM file from raw data
	pub fn generate(&self, data: &[u8], output_path: &Path) -> AssemblyResult<()> {
		// For simple generation, assume it's PRG ROM data
		let header = INesHeader {
			prg_banks: ((data.len() + 16383) / 16384) as u8, // Round up to nearest 16KB
			..INesHeader::default()
		};

		let mut rom_data = RomData::new();
		rom_data.set_prg_rom(data.to_vec());

		// Pad to correct size
		let expected_size = header.prg_size();
		if rom_data.prg_rom.len() < expected_size {
			rom_data.prg_rom.resize(
				expected_size,
				if self.zero_fill {
					0
				} else {
					0xFF
				},
			);
		}

		self.generate_rom(&header, &rom_data, output_path)?;
		Ok(())
	}

	/// Generate ROM file
	fn generate_rom(
		&self,
		header: &INesHeader,
		rom_data: &RomData,
		output_path: &Path,
	) -> AssemblyResult<Vec<u8>> {
		// Validate header and data
		header.validate()?;
		rom_data.validate(header)?;

		match self.format {
			RomFormat::INes | RomFormat::Nes20 => {
				self.generate_ines_rom(header, rom_data, output_path)
			}
			RomFormat::Binary => self.generate_binary_rom(rom_data, output_path),
		}
	}

	/// Generate iNES format ROM
	fn generate_ines_rom(
		&self,
		header: &INesHeader,
		rom_data: &RomData,
		output_path: &Path,
	) -> AssemblyResult<Vec<u8>> {
		let mut output_data = Vec::new();

		// Write header
		output_data.extend_from_slice(&header.to_bytes());

		// Write trainer if present
		if let Some(trainer) = &rom_data.trainer {
			output_data.extend_from_slice(trainer);
		}

		// Write PRG ROM
		output_data.extend_from_slice(&rom_data.prg_rom);

		// Write CHR ROM
		output_data.extend_from_slice(&rom_data.chr_rom);

		// Write to file
		self.write_to_file(&output_data, output_path)?;

		Ok(output_data)
	}

	/// Generate binary format ROM (no header)
	fn generate_binary_rom(
		&self,
		rom_data: &RomData,
		output_path: &Path,
	) -> AssemblyResult<Vec<u8>> {
		let mut output_data = Vec::new();

		// Write only PRG ROM for binary format
		output_data.extend_from_slice(&rom_data.prg_rom);

		// Write to file
		self.write_to_file(&output_data, output_path)?;

		Ok(output_data)
	}

	/// Extract ROM data from memory manager
	fn extract_rom_data(
		&self,
		memory: &MemoryManager,
		config: &INesConfig,
	) -> AssemblyResult<RomData> {
		let mut rom_data = RomData::new();

		// Calculate expected sizes
		let prg_size = config.prg_banks as usize * 16384;
		let chr_size = config.chr_banks as usize * 8192;

		// Initialize with correct sizes
		rom_data.prg_rom.resize(
			prg_size,
			if self.zero_fill {
				0
			} else {
				0xFF
			},
		);
		rom_data.chr_rom.resize(
			chr_size,
			if self.zero_fill {
				0
			} else {
				0xFF
			},
		);

		// Extract PRG ROM data from code sections
		for section in memory.sections().values() {
			match section.section_type {
				SectionType::Code | SectionType::Data | SectionType::RoData => {
					// Map to PRG ROM
					let start_addr = section.start_address;
					if start_addr >= 0x8000 {
						let rom_offset = (start_addr - 0x8000) as usize;
						let end_offset = rom_offset + section.data.len();

						if end_offset <= rom_data.prg_rom.len() {
							rom_data.prg_rom[rom_offset..end_offset].copy_from_slice(&section.data);
						}
					}
				}
				SectionType::ChrRom => {
					// Map to CHR ROM
					let chr_offset = section.start_address as usize;
					let end_offset = chr_offset + section.data.len();

					if end_offset <= rom_data.chr_rom.len() {
						rom_data.chr_rom[chr_offset..end_offset].copy_from_slice(&section.data);
					}
				}
				_ => {
					// Skip other section types (BSS, ZeroPage, etc.)
				}
			}
		}

		Ok(rom_data)
	}

	/// Write data to file
	fn write_to_file(&self, data: &[u8], output_path: &Path) -> AssemblyResult<()> {
		let file = File::create(output_path).map_err(|e| AssemblyError::Io {
			pos: Some(SourcePos::file_only(output_path.to_path_buf())),
			source: e,
		})?;

		let mut writer = BufWriter::new(file);
		writer.write_all(data).map_err(|e| AssemblyError::Io {
			pos: Some(SourcePos::file_only(output_path.to_path_buf())),
			source: e,
		})?;

		writer.flush().map_err(|e| AssemblyError::Io {
			pos: Some(SourcePos::file_only(output_path.to_path_buf())),
			source: e,
		})?;

		Ok(())
	}

	/// Calculate CRC32 checksum (for validation)
	#[cfg(feature = "crc-validation")]
	pub fn calculate_crc32(&self, data: &[u8]) -> u32 {
		use crc::{CRC_32_ISO_HDLC, Crc};
		const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);
		CRC32.checksum(data)
	}

	/// Get ROM information summary
	pub fn get_rom_info(&self, header: &INesHeader, rom_data: &RomData) -> RomInfo {
		RomInfo {
			format: self.format,
			mapper: header.mapper(),
			prg_banks: header.prg_banks,
			chr_banks: header.chr_banks,
			prg_size: rom_data.prg_rom.len(),
			chr_size: rom_data.chr_rom.len(),
			total_size: 16 + rom_data.total_size(), // Header + ROM data
			mirroring: header.mirroring(),
			has_battery: header.has_battery(),
			has_trainer: header.has_trainer(),
		}
	}
}

impl Default for RomGenerator {
	fn default() -> Self {
		Self::new()
	}
}

/// ROM information summary
#[derive(Debug, Clone)]
pub struct RomInfo {
	pub format: RomFormat,
	pub mapper: u8,
	pub prg_banks: u8,
	pub chr_banks: u8,
	pub prg_size: usize,
	pub chr_size: usize,
	pub total_size: usize,
	pub mirroring: MirroringType,
	pub has_battery: bool,
	pub has_trainer: bool,
}

impl std::fmt::Display for RomInfo {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "ROM Information:")?;
		writeln!(f, "  Format: {:?}", self.format)?;
		writeln!(f, "  Mapper: {}", self.mapper)?;
		writeln!(f, "  PRG ROM: {} banks ({} KB)", self.prg_banks, self.prg_size / 1024)?;
		writeln!(f, "  CHR ROM: {} banks ({} KB)", self.chr_banks, self.chr_size / 1024)?;
		writeln!(f, "  Total size: {} bytes", self.total_size)?;
		writeln!(f, "  Mirroring: {:?}", self.mirroring)?;
		writeln!(f, "  Battery: {}", self.has_battery)?;
		writeln!(f, "  Trainer: {}", self.has_trainer)?;
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::path::PathBuf;

	#[test]
	fn test_ines_header_creation() {
		let header = INesHeader::new();
		assert_eq!(header.magic, [b'N', b'E', b'S', 0x1A]);
		assert_eq!(header.prg_banks, 1);
		assert_eq!(header.chr_banks, 0);
	}

	#[test]
	fn test_ines_header_from_config() {
		let config = INesConfig {
			mapper: 1,
			prg_banks: 2,
			chr_banks: 1,
			mirroring: MirroringType::Vertical,
			battery: true,
			trainer: false,
			four_screen: false,
			video_system: VideoSystem::Ntsc,
		};

		let header = INesHeader::from_config(&config);
		assert_eq!(header.prg_banks, 2);
		assert_eq!(header.chr_banks, 1);
		assert_eq!(header.mapper(), 1);
		assert!(header.has_battery());
		assert!(!header.has_trainer());
		assert_eq!(header.mirroring(), MirroringType::Vertical);
	}

	#[test]
	fn test_ines_header_validation() {
		let mut header = INesHeader::new();
		assert!(header.validate().is_ok());

		header.prg_banks = 0;
		assert!(header.validate().is_err());

		header.prg_banks = 50;
		assert!(header.validate().is_err());
	}

	#[test]
	fn test_rom_data_creation() {
		let rom_data = RomData::with_sizes(32768, 8192);
		assert_eq!(rom_data.prg_rom.len(), 32768);
		assert_eq!(rom_data.chr_rom.len(), 8192);
		assert!(rom_data.trainer.is_none());
	}

	#[test]
	fn test_rom_data_with_trainer() {
		let trainer_data = vec![0; 512];
		let rom_data = RomData::new().with_trainer(trainer_data).unwrap();
		assert!(rom_data.trainer.is_some());
		assert_eq!(rom_data.trainer.as_ref().unwrap().len(), 512);

		// Test invalid trainer size
		let invalid_trainer = vec![0; 256];
		assert!(RomData::new().with_trainer(invalid_trainer).is_err());
	}

	#[test]
	fn test_rom_data_validation() {
		let header = INesHeader {
			prg_banks: 2,
			chr_banks: 1,
			..INesHeader::new()
		};

		let rom_data = RomData::with_sizes(32768, 8192); // 2 PRG banks, 1 CHR bank
		assert!(rom_data.validate(&header).is_ok());

		let wrong_size_data = RomData::with_sizes(16384, 8192); // Wrong PRG size
		assert!(wrong_size_data.validate(&header).is_err());
	}

	#[test]
	fn test_rom_generator_creation() {
		let generator = RomGenerator::new().with_format(RomFormat::INes).with_zero_fill(true);

		assert_eq!(generator.format, RomFormat::INes);
		assert!(generator.zero_fill);
	}

	#[test]
	fn test_header_to_bytes() {
		let header = INesHeader {
			magic: [b'N', b'E', b'S', 0x1A],
			prg_banks: 2,
			chr_banks: 1,
			flags6: 0x11, // Vertical mirroring, mapper 1
			flags7: 0x00,
			..INesHeader::new()
		};

		let bytes = header.to_bytes();
		assert_eq!(bytes[0..4], [b'N', b'E', b'S', 0x1A]);
		assert_eq!(bytes[4], 2); // PRG banks
		assert_eq!(bytes[5], 1); // CHR banks
		assert_eq!(bytes[6], 0x11); // Flags6
		assert_eq!(bytes[7], 0x00); // Flags7
	}

	#[test]
	fn test_mapper_extraction() {
		let header = INesHeader {
			flags6: 0x10, // Mapper low nibble = 1
			flags7: 0x20, // Mapper high nibble = 2
			..INesHeader::new()
		};

		assert_eq!(header.mapper(), 0x21); // Should be 0x20 | 0x01 = 33
	}

	#[test]
	fn test_rom_info_display() {
		let info = RomInfo {
			format: RomFormat::INes,
			mapper: 1,
			prg_banks: 2,
			chr_banks: 1,
			prg_size: 32768,
			chr_size: 8192,
			total_size: 40976, // 16 + 32768 + 8192
			mirroring: MirroringType::Vertical,
			has_battery: true,
			has_trainer: false,
		};

		let display = format!("{}", info);
		assert!(display.contains("Mapper: 1"));
		assert!(display.contains("PRG ROM: 2 banks (32 KB)"));
		assert!(display.contains("CHR ROM: 1 banks (8 KB)"));
		assert!(display.contains("Battery: true"));
	}
}
