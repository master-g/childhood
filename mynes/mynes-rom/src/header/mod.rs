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
pub enum NameTableMirroring {
	Vertical,
	Horizontal,
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
	pub name_table_mirroring: NameTableMirroring,

	/// If the cartridge has a battery-backed save RAM
	pub has_battery: bool,

	/// If the cartridge has a trainer
	pub has_trainer: bool,

	/// Mapper number
	pub mapper: u8,

	/// The console type
	pub console_type: ConsoleType,

	/// The size of the PRG RAM in bytes
	pub prg_ram_size: usize,

	/// The size of the PRG-NVRAM/EEPROM in bytes
	pub eeprom_size: Option<usize>,

	/// The size of the CHR RAM in bytes
	pub chr_ram_size: Option<usize>,

	/// The size of the CHR-NVRAM in bytes
	pub chr_nvram_size: Option<usize>,

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
		let name_table_mirroring = match (four_screen, vertical_mirroring) {
			(true, _) => NameTableMirroring::FourScreen,
			(false, true) => NameTableMirroring::Vertical,
			(false, false) => NameTableMirroring::Horizontal,
		};

		let has_battery = raw[6] & 0b10 != 0;
		let has_trainer = raw[6] & 0b100 != 0;

		let mapper = (raw[7] & 0b1111_0000) | (raw[6] >> 4);
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
			name_table_mirroring,
			has_battery,
			has_trainer,
			mapper,
			console_type,
			prg_ram_size,
			eeprom_size: None,
			chr_ram_size: None,
			chr_nvram_size: None,
			timing_mode,
			vs_ppu_type: 0,
			vs_hardware_type: 0,
			extended_console_type: 0,
			num_of_misc_roms: 0,
			default_expansion_device: 0,
		}
	}

	fn extract_version_two(raw: &[u8]) -> Self {
		todo!()
	}
}
