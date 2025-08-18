//! Platform-specific functionality for the NES compiler
//!
//! This module contains platform-specific code for different target systems,
//! currently focusing on the Nintendo Entertainment System (NES).

pub mod nes;

pub use nes::{NesConfig, NesPlatform};

use crate::error::AssemblyResult;

/// Trait for platform-specific functionality
pub trait Platform: std::fmt::Debug {
	/// Get the platform name
	fn name(&self) -> &str;

	/// Get the default ROM extension
	fn rom_extension(&self) -> &str;

	/// Get the default bank size
	fn bank_size(&self) -> usize;

	/// Get the maximum number of banks
	fn max_banks(&self) -> usize;

	/// Validate a memory address for this platform
	fn validate_address(&self, address: u16) -> bool;

	/// Generate platform-specific ROM header
	fn generate_header(&self, prg_banks: u8, chr_banks: u8) -> AssemblyResult<Vec<u8>>;

	/// Get predefined symbols for this platform
	fn predefined_symbols(&self) -> Vec<(String, i32)>;
}

/// Platform types supported by the compiler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformType {
	/// Nintendo Entertainment System
	Nes,
}

impl PlatformType {
	/// Get the platform name as a string
	pub fn name(&self) -> &'static str {
		match self {
			Self::Nes => "NES",
		}
	}

	/// Create a platform instance for this type
	pub fn create_platform(&self) -> Box<dyn Platform> {
		match self {
			Self::Nes => Box::new(NesPlatform::new()),
		}
	}
}

impl Default for PlatformType {
	fn default() -> Self {
		Self::Nes
	}
}

/// Platform manager that handles platform-specific operations
#[derive(Debug)]
pub struct PlatformManager {
	/// Current platform
	platform: Box<dyn Platform>,
	/// Platform type
	platform_type: PlatformType,
}

impl PlatformManager {
	/// Create a new platform manager for the specified platform
	pub fn new(platform_type: PlatformType) -> Self {
		Self {
			platform: platform_type.create_platform(),
			platform_type,
		}
	}

	/// Get the current platform
	pub fn platform(&self) -> &dyn Platform {
		self.platform.as_ref()
	}

	/// Get the platform type
	pub fn platform_type(&self) -> PlatformType {
		self.platform_type
	}

	/// Switch to a different platform
	pub fn switch_platform(&mut self, platform_type: PlatformType) {
		self.platform_type = platform_type;
		self.platform = platform_type.create_platform();
	}
}

impl Default for PlatformManager {
	fn default() -> Self {
		Self::new(PlatformType::default())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_platform_type_name() {
		assert_eq!(PlatformType::Nes.name(), "NES");
	}

	#[test]
	fn test_platform_manager_creation() {
		let manager = PlatformManager::new(PlatformType::Nes);
		assert_eq!(manager.platform_type(), PlatformType::Nes);
		assert_eq!(manager.platform().name(), "NES");
	}

	#[test]
	fn test_platform_switching() {
		let mut manager = PlatformManager::new(PlatformType::Nes);
		assert_eq!(manager.platform_type(), PlatformType::Nes);

		manager.switch_platform(PlatformType::Nes);
		assert_eq!(manager.platform_type(), PlatformType::Nes);
	}
}
