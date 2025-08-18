//! Configuration management for the NES assembler.
//!
//! This module provides configuration options for controlling assembler behavior,
//! output format, and optimization settings.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::{AssemblyError, AssemblyResult};

/// Main configuration for the NES assembler.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
	/// Input file settings
	pub input: InputConfig,

	/// Output file settings
	pub output: OutputConfig,

	/// Assembly behavior settings
	pub assembly: AssemblyConfig,

	/// Platform-specific settings
	pub platform: PlatformConfig,

	/// Optimization settings
	pub optimization: OptimizationConfig,

	/// Debug and logging settings
	pub debug: DebugConfig,

	/// Predefined symbols with integer values
	pub predefined_symbols: std::collections::HashMap<String, i32>,

	/// Predefined symbols with string values
	pub predefined_strings: std::collections::HashMap<String, String>,
}

/// Input file configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
	/// Main assembly source file
	pub source_file: Option<PathBuf>,

	/// Additional include directories
	pub include_dirs: Vec<PathBuf>,

	/// Maximum include depth to prevent infinite recursion
	pub max_include_depth: usize,

	/// Character encoding for source files
	pub encoding: FileEncoding,

	/// Case sensitivity for symbols and labels
	pub case_sensitive: bool,
}

/// Output file configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
	/// Output ROM file path
	pub rom_file: Option<PathBuf>,

	/// Output format
	pub format: OutputFormat,

	/// Generate listing file
	pub listing_file: Option<PathBuf>,

	/// Generate symbol file
	pub symbol_file: Option<PathBuf>,

	/// Generate debug information
	pub debug_file: Option<PathBuf>,

	/// Generate map file
	pub map_file: Option<PathBuf>,
}

/// Assembly behavior configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssemblyConfig {
	/// Maximum number of assembly passes
	pub max_passes: usize,

	/// Allow undefined symbols in final pass
	pub allow_undefined_symbols: bool,

	/// Warn on unused symbols
	pub warn_unused_symbols: bool,

	/// Maximum number of errors before stopping
	pub max_errors: Option<usize>,

	/// Enable macro expansion
	pub enable_macros: bool,

	/// Maximum macro recursion depth
	pub max_macro_depth: usize,

	/// Enable conditional assembly
	pub enable_conditionals: bool,

	/// Maximum conditional nesting depth
	pub max_conditional_depth: usize,
}

/// Platform-specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
	/// Target platform
	pub target: TargetPlatform,

	/// NES-specific configuration
	pub nes: NesConfig,
}

/// NES-specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NesConfig {
	/// iNES header configuration
	pub ines: INesConfig,

	/// Memory mapping configuration
	pub memory: MemoryConfig,

	/// Enable NES 2.0 format
	pub nes2_format: bool,
}

/// iNES header configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct INesConfig {
	/// Mapper number (0-255)
	pub mapper: u8,

	/// Number of 16KB PRG ROM banks
	pub prg_banks: u8,

	/// Number of 8KB CHR ROM banks
	pub chr_banks: u8,

	/// Mirroring type
	pub mirroring: MirroringType,

	/// Battery-backed SRAM
	pub battery: bool,

	/// Trainer present
	pub trainer: bool,

	/// Four-screen VRAM
	pub four_screen: bool,

	/// Video system
	pub video_system: VideoSystem,
}

/// Memory configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
	/// Default PRG ROM bank size
	pub prg_bank_size: usize,

	/// Default CHR ROM bank size
	pub chr_bank_size: usize,

	/// Zero page optimization
	pub optimize_zero_page: bool,

	/// Bank switching support
	pub bank_switching: bool,
}

/// Optimization configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
	/// Enable optimizations
	pub enabled: bool,

	/// Optimize relative branches
	pub optimize_branches: bool,

	/// Optimize zero page addressing
	pub optimize_zero_page: bool,

	/// Remove unused symbols
	pub remove_unused_symbols: bool,

	/// Optimize instruction selection
	pub optimize_instructions: bool,
}

/// Debug and logging configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
	/// Verbose output
	pub verbose: bool,

	/// Debug level
	pub debug_level: DebugLevel,

	/// Generate timing information
	pub timing: bool,

	/// Generate memory usage statistics
	pub memory_stats: bool,

	/// Dump intermediate representations
	pub dump_ir: bool,
}

/// File encoding options.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FileEncoding {
	/// UTF-8 encoding
	Utf8,
	/// ASCII encoding
	Ascii,
	/// Latin-1 encoding
	Latin1,
}

/// Output format options.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OutputFormat {
	/// iNES ROM format (.nes)
	INes,
	/// Raw binary format (.bin)
	Binary,
	/// Intel HEX format (.hex)
	IntelHex,
	/// Motorola S-record format (.s19)
	SRecord,
}

/// Target platform options.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TargetPlatform {
	/// Nintendo Entertainment System
	Nes,
	/// Famicom Disk System
	Fds,
	/// Generic 6502 system
	Generic6502,
}

/// Mirroring type for NES cartridges.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum MirroringType {
	/// Horizontal mirroring
	Horizontal,
	/// Vertical mirroring
	Vertical,
	/// Four-screen mirroring
	FourScreen,
	/// Single-screen mirroring
	SingleScreen,
}

/// Video system type.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum VideoSystem {
	/// NTSC (North America, Japan)
	Ntsc,
	/// PAL (Europe, Australia)
	Pal,
	/// Dual compatible
	Dual,
}

/// Debug level options.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DebugLevel {
	/// No debug output
	None,
	/// Basic debug information
	Basic,
	/// Detailed debug information
	Detailed,
	/// Verbose debug information
	Verbose,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			input: InputConfig::default(),
			output: OutputConfig::default(),
			assembly: AssemblyConfig::default(),
			platform: PlatformConfig::default(),
			optimization: OptimizationConfig::default(),
			debug: DebugConfig::default(),
			predefined_symbols: std::collections::HashMap::new(),
			predefined_strings: std::collections::HashMap::new(),
		}
	}
}

impl Default for InputConfig {
	fn default() -> Self {
		Self {
			source_file: None,
			include_dirs: vec![PathBuf::from(".")],
			max_include_depth: 16,
			encoding: FileEncoding::Utf8,
			case_sensitive: false,
		}
	}
}

impl Default for OutputConfig {
	fn default() -> Self {
		Self {
			rom_file: None,
			format: OutputFormat::INes,
			listing_file: None,
			symbol_file: None,
			debug_file: None,
			map_file: None,
		}
	}
}

impl Default for AssemblyConfig {
	fn default() -> Self {
		Self {
			max_passes: 3,
			allow_undefined_symbols: false,
			warn_unused_symbols: true,
			max_errors: Some(100),
			enable_macros: true,
			max_macro_depth: 64,
			enable_conditionals: true,
			max_conditional_depth: 32,
		}
	}
}

impl Default for PlatformConfig {
	fn default() -> Self {
		Self {
			target: TargetPlatform::Nes,
			nes: NesConfig::default(),
		}
	}
}

impl Default for NesConfig {
	fn default() -> Self {
		Self {
			ines: INesConfig::default(),
			memory: MemoryConfig::default(),
			nes2_format: false,
		}
	}
}

impl Default for INesConfig {
	fn default() -> Self {
		Self {
			mapper: 0,
			prg_banks: 1,
			chr_banks: 1,
			mirroring: MirroringType::Horizontal,
			battery: false,
			trainer: false,
			four_screen: false,
			video_system: VideoSystem::Ntsc,
		}
	}
}

impl Default for MemoryConfig {
	fn default() -> Self {
		Self {
			prg_bank_size: 16384, // 16KB
			chr_bank_size: 8192,  // 8KB
			optimize_zero_page: true,
			bank_switching: false,
		}
	}
}

impl Default for OptimizationConfig {
	fn default() -> Self {
		Self {
			enabled: true,
			optimize_branches: true,
			optimize_zero_page: true,
			remove_unused_symbols: false,
			optimize_instructions: true,
		}
	}
}

impl Default for DebugConfig {
	fn default() -> Self {
		Self {
			verbose: false,
			debug_level: DebugLevel::None,
			timing: false,
			memory_stats: false,
			dump_ir: false,
		}
	}
}

impl Config {
	/// Create a new configuration with default values.
	pub fn new() -> Self {
		Self::default()
	}

	/// Load configuration from a file.
	pub fn from_file(path: &std::path::Path) -> AssemblyResult<Self> {
		let content = std::fs::read_to_string(path).map_err(|e| AssemblyError::Io {
			pos: None,
			source: e,
		})?;

		let config: Config = if path.extension().and_then(|s| s.to_str()) == Some("toml") {
			toml::from_str(&content)
				.map_err(|e| AssemblyError::config(format!("Invalid TOML: {}", e)))?
		} else {
			serde_json::from_str(&content)
				.map_err(|e| AssemblyError::config(format!("Invalid JSON: {}", e)))?
		};

		config.validate()?;
		Ok(config)
	}

	/// Save configuration to a file.
	pub fn to_file(&self, path: &std::path::Path) -> AssemblyResult<()> {
		let content = if path.extension().and_then(|s| s.to_str()) == Some("toml") {
			toml::to_string_pretty(self)
				.map_err(|e| AssemblyError::config(format!("Failed to serialize TOML: {}", e)))?
		} else {
			serde_json::to_string_pretty(self)
				.map_err(|e| AssemblyError::config(format!("Failed to serialize JSON: {}", e)))?
		};

		std::fs::write(path, content).map_err(|e| AssemblyError::Io {
			pos: None,
			source: e,
		})?;

		Ok(())
	}

	/// Validate the configuration.
	pub fn validate(&self) -> AssemblyResult<()> {
		// Validate assembly configuration
		if self.assembly.max_passes == 0 {
			return Err(AssemblyError::config("max_passes must be greater than 0"));
		}

		if self.assembly.max_passes > 10 {
			return Err(AssemblyError::config("max_passes must not exceed 10"));
		}

		if self.assembly.max_macro_depth == 0 {
			return Err(AssemblyError::config("max_macro_depth must be greater than 0"));
		}

		if self.assembly.max_conditional_depth == 0 {
			return Err(AssemblyError::config("max_conditional_depth must be greater than 0"));
		}

		// Validate input configuration
		if self.input.max_include_depth == 0 {
			return Err(AssemblyError::config("max_include_depth must be greater than 0"));
		}

		// Validate NES configuration
		if self.platform.nes.ines.prg_banks == 0 {
			return Err(AssemblyError::config("prg_banks must be greater than 0"));
		}

		if self.platform.nes.memory.prg_bank_size == 0 {
			return Err(AssemblyError::config("prg_bank_size must be greater than 0"));
		}

		if self.platform.nes.memory.chr_bank_size == 0 {
			return Err(AssemblyError::config("chr_bank_size must be greater than 0"));
		}

		Ok(())
	}

	/// Set the input file.
	pub fn with_input_file(mut self, path: PathBuf) -> Self {
		self.input.source_file = Some(path);
		self
	}

	/// Set the output file.
	pub fn with_output_file(mut self, path: PathBuf) -> Self {
		self.output.rom_file = Some(path);
		self
	}

	/// Set the output format.
	pub fn with_format(mut self, format: OutputFormat) -> Self {
		self.output.format = format;
		self
	}

	/// Enable verbose output.
	pub fn with_verbose(mut self, verbose: bool) -> Self {
		self.debug.verbose = verbose;
		self
	}

	/// Set the target platform.
	pub fn with_target(mut self, target: TargetPlatform) -> Self {
		self.platform.target = target;
		self
	}

	/// Add an include directory.
	pub fn add_include_dir(mut self, path: PathBuf) -> Self {
		self.input.include_dirs.push(path);
		self
	}

	/// Set mapper number.
	pub fn with_mapper(mut self, mapper: u8) -> Self {
		self.platform.nes.ines.mapper = mapper;
		self
	}

	/// Set PRG ROM banks.
	pub fn with_prg_banks(mut self, banks: u8) -> Self {
		self.platform.nes.ines.prg_banks = banks;
		self
	}

	/// Set CHR ROM banks.
	pub fn with_chr_banks(mut self, banks: u8) -> Self {
		self.platform.nes.ines.chr_banks = banks;
		self
	}

	/// Set mirroring type.
	pub fn with_mirroring(mut self, mirroring: MirroringType) -> Self {
		self.platform.nes.ines.mirroring = mirroring;
		self
	}

	/// Enable optimizations.
	pub fn with_optimizations(mut self, enabled: bool) -> Self {
		self.optimization.enabled = enabled;
		self
	}

	/// Get effective include directories (including current directory).
	pub fn include_directories(&self) -> &[PathBuf] {
		&self.input.include_dirs
	}

	/// Check if a feature is enabled.
	pub fn is_feature_enabled(&self, feature: &str) -> bool {
		match feature {
			"macros" => self.assembly.enable_macros,
			"conditionals" => self.assembly.enable_conditionals,
			"optimizations" => self.optimization.enabled,
			"zero_page_optimization" => self.optimization.optimize_zero_page,
			"branch_optimization" => self.optimization.optimize_branches,
			"instruction_optimization" => self.optimization.optimize_instructions,
			"verbose" => self.debug.verbose,
			"timing" => self.debug.timing,
			"memory_stats" => self.debug.memory_stats,
			_ => false,
		}
	}

	/// Get configuration value by key.
	pub fn get_value(&self, key: &str) -> Option<String> {
		match key {
			"max_passes" => Some(self.assembly.max_passes.to_string()),
			"max_errors" => self.assembly.max_errors.map(|v| v.to_string()),
			"max_macro_depth" => Some(self.assembly.max_macro_depth.to_string()),
			"max_conditional_depth" => Some(self.assembly.max_conditional_depth.to_string()),
			"max_include_depth" => Some(self.input.max_include_depth.to_string()),
			"mapper" => Some(self.platform.nes.ines.mapper.to_string()),
			"prg_banks" => Some(self.platform.nes.ines.prg_banks.to_string()),
			"chr_banks" => Some(self.platform.nes.ines.chr_banks.to_string()),
			"prg_bank_size" => Some(self.platform.nes.memory.prg_bank_size.to_string()),
			"chr_bank_size" => Some(self.platform.nes.memory.chr_bank_size.to_string()),
			_ => None,
		}
	}
}

/// Builder for creating configurations.
#[derive(Debug, Default)]
pub struct ConfigBuilder {
	config: Config,
}

impl ConfigBuilder {
	/// Create a new configuration builder.
	pub fn new() -> Self {
		Self::default()
	}

	/// Set input file.
	pub fn input_file(mut self, path: PathBuf) -> Self {
		self.config.input.source_file = Some(path);
		self
	}

	/// Set output file.
	pub fn output_file(mut self, path: PathBuf) -> Self {
		self.config.output.rom_file = Some(path);
		self
	}

	/// Set output format.
	pub fn format(mut self, format: OutputFormat) -> Self {
		self.config.output.format = format;
		self
	}

	/// Set verbose mode.
	pub fn verbose(mut self, verbose: bool) -> Self {
		self.config.debug.verbose = verbose;
		self
	}

	/// Set target platform.
	pub fn target(mut self, target: TargetPlatform) -> Self {
		self.config.platform.target = target;
		self
	}

	/// Add include directory.
	pub fn include_dir(mut self, path: PathBuf) -> Self {
		self.config.input.include_dirs.push(path);
		self
	}

	/// Set mapper.
	pub fn mapper(mut self, mapper: u8) -> Self {
		self.config.platform.nes.ines.mapper = mapper;
		self
	}

	/// Set PRG banks.
	pub fn prg_banks(mut self, banks: u8) -> Self {
		self.config.platform.nes.ines.prg_banks = banks;
		self
	}

	/// Set CHR banks.
	pub fn chr_banks(mut self, banks: u8) -> Self {
		self.config.platform.nes.ines.chr_banks = banks;
		self
	}

	/// Build the configuration.
	pub fn build(self) -> AssemblyResult<Config> {
		self.config.validate()?;
		Ok(self.config)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::path::Path;

	#[test]
	fn test_default_config() {
		let config = Config::default();
		assert_eq!(config.assembly.max_passes, 3);
		assert_eq!(config.platform.nes.ines.mapper, 0);
		assert!(!config.debug.verbose);
	}

	#[test]
	fn test_config_validation() {
		let mut config = Config::default();
		assert!(config.validate().is_ok());

		config.assembly.max_passes = 0;
		assert!(config.validate().is_err());
	}

	#[test]
	fn test_config_builder() {
		let config = ConfigBuilder::new()
			.input_file(PathBuf::from("test.asm"))
			.output_file(PathBuf::from("test.nes"))
			.verbose(true)
			.mapper(1)
			.build()
			.unwrap();

		assert_eq!(config.input.source_file, Some(PathBuf::from("test.asm")));
		assert_eq!(config.output.rom_file, Some(PathBuf::from("test.nes")));
		assert!(config.debug.verbose);
		assert_eq!(config.platform.nes.ines.mapper, 1);
	}

	#[test]
	fn test_feature_flags() {
		let config = Config::default();
		assert!(config.is_feature_enabled("macros"));
		assert!(config.is_feature_enabled("conditionals"));
		assert!(config.is_feature_enabled("optimizations"));
		assert!(!config.is_feature_enabled("verbose"));
	}

	#[test]
	fn test_config_values() {
		let config = Config::default();
		assert_eq!(config.get_value("max_passes"), Some("3".to_string()));
		assert_eq!(config.get_value("mapper"), Some("0".to_string()));
		assert_eq!(config.get_value("nonexistent"), None);
	}

	#[test]
	fn test_fluent_interface() {
		let config = Config::new()
			.with_input_file(PathBuf::from("test.asm"))
			.with_output_file(PathBuf::from("test.nes"))
			.with_verbose(true)
			.with_mapper(2)
			.with_optimizations(false);

		assert_eq!(config.input.source_file, Some(PathBuf::from("test.asm")));
		assert!(config.debug.verbose);
		assert_eq!(config.platform.nes.ines.mapper, 2);
		assert!(!config.optimization.enabled);
	}
}
