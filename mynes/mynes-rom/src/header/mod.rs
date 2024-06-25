use std::ops::Shl;

use serde::{Deserialize, Serialize};

use crate::err::Error;

const INES_TAG: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];
const INES_HEADER_SIZE: usize = 16;
const PRG_ROM_PAGE_SIZE: usize = 16384;
const CHR_ROM_PAGE_SIZE: usize = 8192;
const PRG_RAM_PAGE_SIZE: usize = 8192;
const TRAINER_ROM_SIZE: usize = 512;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Version {
	One,
	Two,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum NameTableLayout {
	VerticalMirroring,
	HorizontalMirroring,
	FourScreen,
	MapperControlled,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum ConsoleType {
	Nes,
	VsSystem,
	Playchoice10,
	Extended,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum TimingMode {
	Ntsc,
	Pal,
	MultipleRegion,
	Ua6538,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]

pub struct INesHeaderInfo {
	/// The version of the iNES format
	pub version: Version,

	/// The size of the PRG ROM in bytes
	pub prg_rom_size: usize,

	/// The size of the CHR ROM in bytes
	pub chr_rom_size: usize,

	/// The hard-wired name table layout
	pub name_table_layout: NameTableLayout,

	/// If the cartridge has a battery-backed save RAM
	pub has_battery: bool,

	/// If the cartridge has a trainer
	pub has_trainer: bool,

	/// Mapper number
	pub mapper: usize,

	/// Sub-mapper number
	pub sub_mapper: usize,

	/// The console type
	pub console_type: ConsoleType,

	/// The size of the PRG RAM in bytes
	pub prg_ram_size: usize,

	/// The size of the PRG-NVRAM/EEPROM in bytes
	pub eeprom_size: usize,

	/// The size of the CHR RAM in bytes
	pub chr_ram_size: usize,

	/// The size of the CHR-NVRAM in bytes
	pub chr_nvram_size: usize,

	/// The CPU/PPU timing mode
	pub timing_mode: TimingMode,

	/// The VS Unisystem PPU type
	pub vs_ppu_type: u8,

	/// The VS Unisystem hardware type
	pub vs_hardware_type: u8,

	/// The extended console type
	pub extended_console_type: u8,

	/// The number of miscellaneous ROMs
	pub num_of_misc_roms: u8,

	/// The default expansion device
	pub default_expansion_device: u8,
}

impl INesHeaderInfo {
	/// Parse the iNES header from a byte slice
	///
	/// # Arguments
	///
	/// * `raw` - The byte slice containing the iNES header
	///
	/// # Returns
	///
	/// The parsed iNES header
	///
	/// # Errors
	///
	/// Returns an error if the header is invalid
	pub fn new(raw: &[u8]) -> Result<Self, Error> {
		if raw.len() < INES_HEADER_SIZE {
			return Err(Error::InvalidHeader);
		}
		if raw[0..4] != INES_TAG {
			return Err(Error::InvalidHeader);
		}
		match (raw[7] >> 2) & 0b11 {
			0 => Ok(Self::extract_verion_one(raw)),
			2 => Ok(Self::extract_version_two(raw)),
			_ => Err(Error::UnsupportedVersion),
		}
	}

	#[allow(clippy::similar_names)]
	fn extract_verion_one(raw: &[u8]) -> Self {
		let prg_rom_size = raw[4] as usize * PRG_ROM_PAGE_SIZE;
		let chr_rom_size = raw[5] as usize * CHR_ROM_PAGE_SIZE;

		let four_screen = raw[6] & 0b1000 != 0;
		let vertical_mirroring = raw[6] & 0b1 != 0;
		let name_table_layout = match (four_screen, vertical_mirroring) {
			(true, _) => NameTableLayout::FourScreen,
			(false, true) => NameTableLayout::VerticalMirroring,
			(false, false) => NameTableLayout::HorizontalMirroring,
		};

		let has_battery = raw[6] & 0b10 != 0;
		let has_trainer = raw[6] & 0b100 != 0;

		let mapper = usize::from((raw[7] & 0b1111_0000) | (raw[6] >> 4));
		let console_type = match raw[7] & 0b11 {
			0 => ConsoleType::Nes,
			1 => ConsoleType::VsSystem,
			2 => ConsoleType::Playchoice10,
			3 => ConsoleType::Extended,
			_ => unreachable!(),
		};

		let prg_ram_size = raw[8] as usize * PRG_RAM_PAGE_SIZE;
		let prg_ram_size = if prg_ram_size == 0 {
			PRG_RAM_PAGE_SIZE
		} else {
			prg_ram_size
		};

		let timing_mode = match raw[9] & 0b1 {
			0 => TimingMode::Ntsc,
			1 => TimingMode::Pal,
			_ => unreachable!(),
		};

		Self {
			version: Version::One,
			prg_rom_size,
			chr_rom_size,
			name_table_layout,
			has_battery,
			has_trainer,
			mapper,
			sub_mapper: 0,
			console_type,
			prg_ram_size,
			eeprom_size: 0,
			chr_ram_size: 0,
			chr_nvram_size: 0,
			timing_mode,
			vs_ppu_type: 0,
			vs_hardware_type: 0,
			extended_console_type: 0,
			num_of_misc_roms: 0,
			default_expansion_device: 0,
		}
	}

	fn cal_rom_size(msb: u8, lsb: u8, page_size: usize) -> usize {
		if msb == 0xF {
			let flags = usize::from(lsb) | (usize::from(msb) << 8);
			let multipler = (flags & 0b0011) * 2 + 1;
			let exponent = flags & 0b0000_1111_1100;
			(2 << exponent) * multipler
		} else {
			(usize::from(lsb) + (usize::from(msb) << 8)) * page_size
		}
	}

	#[allow(clippy::similar_names)]
	fn extract_version_two(raw: &[u8]) -> Self {
		let prg_rom_lsb = raw[4];
		let prg_rom_msb = raw[9] & 0b0000_1111;
		let prg_rom_size = Self::cal_rom_size(prg_rom_msb, prg_rom_lsb, PRG_ROM_PAGE_SIZE);

		let chr_rom_lsb = raw[5];
		let chr_rom_msb = raw[9] >> 4;
		let chr_rom_size = Self::cal_rom_size(chr_rom_msb, chr_rom_lsb, CHR_ROM_PAGE_SIZE);

		let name_table_lsb = raw[6] & 0b1;
		let name_table_msb = (raw[6] & 0b1000) >> 3;

		let name_table_layout = match (name_table_msb, name_table_lsb) {
			(0, 0) => NameTableLayout::VerticalMirroring,
			(0, 1) => NameTableLayout::HorizontalMirroring,
			(1, _) => NameTableLayout::MapperControlled,
			_ => unreachable!(),
		};

		let has_battery = raw[6] & 0b10 != 0;
		let has_trainer = raw[6] & 0b100 != 0;

		let mapper_low = usize::from(raw[6] >> 4 | (raw[7] & 0b1111_0000));
		let mapper_msb = usize::from(raw[8] & 0b0000_1111).shl(8);
		let mapper = mapper_low | mapper_msb;

		let sub_mapper = usize::from(raw[8] & 0b1111_0000) >> 4;

		let console_type = match raw[7] & 0b11 {
			0 => ConsoleType::Nes,
			1 => ConsoleType::VsSystem,
			2 => ConsoleType::Playchoice10,
			3 => ConsoleType::Extended,
			_ => unreachable!(),
		};

		let timing_mode = match raw[12] & 0b11 {
			0 => TimingMode::Ntsc,
			1 => TimingMode::Pal,
			2 => TimingMode::MultipleRegion,
			3 => TimingMode::Ua6538,
			_ => unreachable!(),
		};

		let shift_count = raw[10] & 0b0000_1111;
		let prg_ram_size = if shift_count == 0 {
			0
		} else {
			64 << shift_count
		};

		let shift_count = raw[10] >> 4;
		let eeprom_size = if shift_count == 0 {
			0
		} else {
			64 << shift_count
		};

		let shift_count = raw[11] & 0b0000_1111;
		let chr_ram_size = if shift_count == 0 {
			0
		} else {
			64 << shift_count
		};

		let shift_count = raw[11] >> 4;
		let chr_nvram_size = if shift_count == 0 {
			0
		} else {
			64 << shift_count
		};

		let vs_ppu_type = if console_type == ConsoleType::VsSystem {
			raw[13] & 0b0000_1111
		} else {
			0
		};
		let vs_hardware_type = if console_type == ConsoleType::VsSystem {
			raw[13] >> 4
		} else {
			0
		};

		let extended_console_type = if console_type == ConsoleType::Extended {
			raw[13] & 0b0000_1111
		} else {
			0
		};

		let num_of_misc_roms = raw[14] & 0b0000_0011;

		let default_expansion_device = raw[15] & 0b0011_1111;

		Self {
			version: Version::Two,
			prg_rom_size,
			chr_rom_size,
			name_table_layout,
			has_battery,
			has_trainer,
			mapper,
			sub_mapper,
			console_type,
			prg_ram_size,
			eeprom_size,
			chr_ram_size,
			chr_nvram_size,
			timing_mode,
			vs_ppu_type,
			vs_hardware_type,
			extended_console_type,
			num_of_misc_roms,
			default_expansion_device,
		}
	}
}
