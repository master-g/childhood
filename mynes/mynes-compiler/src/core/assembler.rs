//! Main assembler engine for the NES compiler.
//!
//! This module provides the core assembler functionality including multi-pass
//! assembly, symbol resolution, instruction encoding, and ROM generation.

use crate::config::Config;
use crate::core::memory::{MemoryManager, SectionType};
use crate::core::passes::{PassManager, PassResult};
use crate::error::{AssemblyError, AssemblyResult, ErrorCollector, SourcePos};
use crate::instructions::CompleteInstruction;
use crate::parsing::{Parser, Statement};
use crate::symbols::{SymbolManager, SymbolType, SymbolValue, SymbolVisibility};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Assembler state during assembly process
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssemblerState {
	/// Initial state
	Initial,
	/// Currently parsing source files
	Parsing,
	/// Running assembly passes
	Assembling,
	/// Generating output
	Generating,
	/// Assembly completed successfully
	Complete,
	/// Assembly failed with errors
	Failed,
}

/// Statistics about the assembly process
#[derive(Debug, Clone, Default)]
pub struct AssemblyStats {
	/// Number of source files processed
	pub files_processed: usize,
	/// Number of lines processed
	pub lines_processed: usize,
	/// Number of statements processed
	pub statements_processed: usize,
	/// Number of instructions assembled
	pub instructions_assembled: usize,
	/// Number of bytes generated
	pub bytes_generated: usize,
	/// Number of assembly passes performed
	pub passes_performed: usize,
	/// Number of errors encountered
	pub errors_encountered: usize,
	/// Number of warnings generated
	pub warnings_generated: usize,
	/// Assembly time in milliseconds
	pub assembly_time_ms: u64,
}

impl AssemblyStats {
	/// Create new statistics
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	/// Reset all statistics
	pub fn reset(&mut self) {
		*self = Self::default();
	}

	/// Add statistics from another instance
	pub fn add(&mut self, other: &AssemblyStats) {
		self.files_processed += other.files_processed;
		self.lines_processed += other.lines_processed;
		self.statements_processed += other.statements_processed;
		self.instructions_assembled += other.instructions_assembled;
		self.bytes_generated += other.bytes_generated;
		self.passes_performed += other.passes_performed;
		self.errors_encountered += other.errors_encountered;
		self.warnings_generated += other.warnings_generated;
		self.assembly_time_ms += other.assembly_time_ms;
	}
}

/// Main assembler engine
pub struct Assembler {
	/// Configuration
	config: Config,
	/// Current assembler state
	state: AssemblerState,
	/// Memory manager
	memory: MemoryManager,
	/// Symbol manager
	symbols: SymbolManager,
	/// Pass manager
	pass_manager: PassManager,
	/// Parser
	parser: Parser,
	/// Source files and their statements
	source_files: HashMap<PathBuf, Vec<Statement>>,
	/// Error collector
	errors: ErrorCollector,
	/// Assembly statistics
	stats: AssemblyStats,
	/// Current program counter
	pc: u16,
	/// Current bank
	current_bank: Option<usize>,
	/// Current section
	current_section: Option<String>,
	/// Platform-specific configuration
	platform_config: Option<Box<dyn std::any::Any>>,
}

impl Assembler {
	/// Create a new assembler with the given configuration
	#[must_use]
	pub fn new(config: Config) -> Self {
		let memory = MemoryManager::new(0xFF);
		let symbols = SymbolManager::new();
		let pass_manager = PassManager::new();
		let parser = Parser::new();
		let errors = if let Some(max) = config.assembly.max_errors {
			ErrorCollector::with_max_errors(max)
		} else {
			ErrorCollector::new()
		};

		Self {
			config,
			state: AssemblerState::Initial,
			memory,
			symbols,
			pass_manager,
			parser,
			source_files: HashMap::new(),
			errors,
			stats: AssemblyStats::new(),
			pc: 0,
			current_bank: None,
			current_section: None,
			platform_config: None,
		}
	}

	/// Assemble a source file
	pub fn assemble_file<P: AsRef<Path>>(&mut self, path: P) -> AssemblyResult<Vec<u8>> {
		let start_time = std::time::Instant::now();
		self.state = AssemblerState::Parsing;
		self.stats.reset();

		// Read and parse the source file
		let path = path.as_ref().to_path_buf();
		self.load_source_file(&path)?;

		// Set up initial memory layout
		self.setup_memory_layout()?;

		// Run assembly passes
		self.state = AssemblerState::Assembling;
		self.run_assembly_passes()?;

		// Generate output
		self.state = AssemblerState::Generating;
		let rom_data = self.generate_output();

		// Update statistics
		#[allow(clippy::cast_possible_truncation)]
		{
			self.stats.assembly_time_ms = start_time.elapsed().as_millis() as u64;
		}
		self.state = AssemblerState::Complete;

		Ok(rom_data)
	}

	/// Load and parse a source file
	fn load_source_file(&mut self, path: &Path) -> AssemblyResult<()> {
		// Read file content
		let content = std::fs::read_to_string(path).map_err(|e| AssemblyError::Io {
			pos: None,
			source: e,
		})?;

		// Parse the content
		let statements = self.parser.parse_file(&content, path.to_string_lossy().to_string())?;

		// Update statistics
		self.stats.files_processed += 1;
		self.stats.lines_processed += content.lines().count();
		self.stats.statements_processed += statements.len();

		// Store parsed statements
		self.source_files.insert(path.to_path_buf(), statements);

		Ok(())
	}

	/// Set up initial memory layout based on configuration
	fn setup_memory_layout(&mut self) -> AssemblyResult<()> {
		let config = &self.config;
		let pos = SourcePos::file_only(PathBuf::from("<config>"));

		// Create PRG ROM sections
		if config.platform.nes.ines.prg_banks > 0 {
			let bank_size = config.platform.nes.memory.prg_bank_size;
			for bank in 0..config.platform.nes.ines.prg_banks {
				let start_addr = 0x8000 + (bank as u16 * 0x4000);
				self.memory.create_section(
					format!("PRG_ROM_{}", bank),
					SectionType::Code,
					start_addr,
					bank_size as u16,
					true,
					&pos,
				)?;
			}
		}

		// Create CHR ROM sections
		if config.platform.nes.ines.chr_banks > 0 {
			let bank_size = config.platform.nes.memory.chr_bank_size;
			for bank in 0..config.platform.nes.ines.chr_banks {
				self.memory.create_section(
					format!("CHR_ROM_{}", bank),
					SectionType::ChrRom,
					bank as u16 * 0x1000,
					bank_size as u16,
					true,
					&pos,
				)?;
			}
		}

		// Create RAM sections
		self.memory.create_section(
			"ZEROPAGE".to_string(),
			SectionType::ZeroPage,
			0x0000,
			0x0100,
			true,
			&pos,
		)?;

		self.memory.create_section(
			"RAM".to_string(),
			SectionType::Data,
			0x0200,
			0x0600,
			true,
			&pos,
		)?;

		// Set default section
		self.current_section = Some("PRG_ROM_0".to_string());
		self.memory.set_current_section("PRG_ROM_0", &pos)?;
		self.pc = 0x8000;

		Ok(())
	}

	/// Run all assembly passes
	fn run_assembly_passes(&mut self) -> AssemblyResult<()> {
		for pass_num in 1..=self.config.assembly.max_passes {
			self.stats.passes_performed = pass_num;

			// Run the pass
			let pass_result = self.run_assembly_pass(pass_num)?;

			match pass_result {
				PassResult::Complete => break,
				PassResult::NeedsAnotherPass => {
					if pass_num >= self.config.assembly.max_passes {
						return Err(AssemblyError::internal(
							None,
							format!(
								"Maximum passes ({}) exceeded",
								self.config.assembly.max_passes
							),
						));
					}
				}
				PassResult::Continue => {}
			}
		}

		// Resolve any remaining forward references
		self.symbols.resolve_forward_references()?;

		// Check for undefined symbols
		let undefined = self.symbols.undefined_symbols();
		if !undefined.is_empty() && !self.config.assembly.allow_undefined_symbols {
			for symbol in undefined {
				self.errors.add(AssemblyError::undefined_symbol(
					SourcePos::file_only(PathBuf::from("<unknown>")),
					symbol,
				));
			}
		}

		// Return any collected errors
		self.errors.to_result(())
	}

	/// Run a single assembly pass
	fn run_assembly_pass(&mut self, pass_num: usize) -> AssemblyResult<PassResult> {
		let mut forward_refs_resolved = 0;
		let mut instructions_processed = 0;

		// Reset program counter and memory positions for this pass
		self.pc = 0x8000;
		if let Some(section_name) = &self.current_section.clone() {
			let pos = SourcePos::file_only(PathBuf::from("<assembler>"));
			if let Some(section) = self.memory.get_section_mut(section_name) {
				section.set_position(section.start_address, &pos)?;
			}
		}

		// Process all statements in all source files
		let source_files = self.source_files.clone();
		for (_file_path, statements) in &source_files {
			for statement in statements {
				let result = self.process_statement(statement, pass_num);

				match result {
					Ok(()) => {
						if statement.is_instruction() {
							instructions_processed += 1;
						}
					}
					Err(err) => {
						// Some errors are expected in early passes (forward references)
						if pass_num < self.config.assembly.max_passes {
							match &err {
								AssemblyError::UndefinedSymbol {
									..
								} => {
									// Expected in early passes - add as forward reference
									forward_refs_resolved += 1;
									// Expected in early passes - skip this error
								}
								_ => {
									if self.errors.add(err) {
										return Err(AssemblyError::multiple(
											self.errors.errors().to_vec(),
										));
									}
								}
							}
						} else if self.errors.add(err) {
							return Err(AssemblyError::multiple(self.errors.errors().to_vec()));
						}
					}
				}
			}
		}

		// Update statistics
		self.stats.instructions_assembled = instructions_processed;

		// Determine if we need another pass
		let undefined_count = self.symbols.undefined_symbols().len();
		if undefined_count == 0 && self.errors.is_empty() {
			Ok(PassResult::Complete)
		} else if pass_num == 1 || forward_refs_resolved > 0 {
			Ok(PassResult::NeedsAnotherPass)
		} else {
			Ok(PassResult::Continue)
		}
	}

	/// Process a single statement
	fn process_statement(&mut self, statement: &Statement, pass_num: usize) -> AssemblyResult<()> {
		match statement {
			Statement::Label {
				name,
				pos,
			} => {
				self.process_label(name, pos)?;
			}
			Statement::Instruction(instruction) => {
				self.process_instruction(instruction, pass_num)?;
			}
			Statement::Directive(directive) => {
				self.process_directive(directive)?;
			}
			Statement::Empty => {
				// Nothing to do
			}
		}
		Ok(())
	}

	/// Process a label definition
	fn process_label(&mut self, name: &str, pos: &SourcePos) -> AssemblyResult<()> {
		// Define the label with current PC value
		self.symbols.define_symbol(
			name.to_string(),
			SymbolType::Label,
			SymbolValue::Address(self.pc),
			pos.clone(),
			SymbolVisibility::Global,
		)?;

		// Update current label context for local labels
		// Note: scope_manager returns immutable reference, we need mutable access
		// This would need to be redesigned to allow mutable access
		// For now, we'll skip this operation

		Ok(())
	}

	/// Process an instruction
	fn process_instruction(
		&mut self,
		instruction: &CompleteInstruction,
		pass_num: usize,
	) -> AssemblyResult<()> {
		// Validate the instruction
		instruction.validate()?;

		// Encode the instruction if possible
		if !instruction.needs_resolution() || pass_num >= self.config.assembly.max_passes {
			let bytes = instruction.encode()?;

			// Write to memory
			self.memory.write_data(&bytes, &instruction.source_pos)?;

			// Update program counter
			self.pc = self.pc.wrapping_add(bytes.len() as u16);

			// Update statistics
			self.stats.bytes_generated += bytes.len();
		} else {
			// Reserve space for forward references
			let size = instruction.size();
			let pos = &instruction.source_pos;
			if let Some(section) = self.memory.current_section_mut() {
				section.advance(size as u16, pos)?;
			}
			self.pc = self.pc.wrapping_add(size as u16);
		}

		Ok(())
	}

	/// Process an assembler directive
	fn process_directive(
		&mut self,
		directive: &crate::parsing::directives::Directive,
	) -> AssemblyResult<()> {
		use crate::parsing::directives::DirectiveType;

		match &directive.directive_type {
			DirectiveType::Org {
				address,
			} => {
				self.pc = *address;
				if let Some(section) = self.memory.current_section_mut() {
					section.set_position(*address, &directive.pos)?;
				}
			}
			DirectiveType::Section {
				name,
			} => {
				self.current_section = Some(name.clone());
				self.memory.set_current_section(name, &directive.pos)?;
			}
			DirectiveType::DataByte {
				data,
			} => {
				let bytes: Vec<u8> = data.iter().map(|&v| v).collect();
				self.memory.write_data(&bytes, &directive.pos)?;
				#[allow(clippy::cast_possible_truncation)]
				{
					self.pc = self.pc.wrapping_add(bytes.len() as u16);
				}
				self.stats.bytes_generated += bytes.len();
			}
			DirectiveType::DataWord {
				data,
			} => {
				let mut bytes = Vec::new();
				for &value in data {
					bytes.extend_from_slice(&value.to_le_bytes());
				}
				self.memory.write_data(&bytes, &directive.pos)?;
				#[allow(clippy::cast_possible_truncation)]
				{
					self.pc = self.pc.wrapping_add(bytes.len() as u16);
				}
				self.stats.bytes_generated += bytes.len();
			}
			DirectiveType::Reserve {
				size,
			} => {
				if let Some(section) = self.memory.current_section_mut() {
					#[allow(clippy::cast_possible_truncation)]
					{
						section.advance(*size as u16, &directive.pos)?;
					}
				}
				#[allow(clippy::cast_possible_truncation)]
				{
					self.pc = self.pc.wrapping_add(*size as u16);
				}
			}
			DirectiveType::Align {
				boundary,
			} => {
				let alignment = u16::from(*boundary);
				let aligned_pc = (self.pc + alignment - 1) & !(alignment - 1);
				let padding = aligned_pc - self.pc;

				if padding > 0 {
					if let Some(section) = self.memory.current_section_mut() {
						section.advance(padding, &directive.pos)?;
					}
					self.pc = aligned_pc;
				}
			}
			DirectiveType::Equ {
				name,
				value,
			} => {
				self.symbols.define_symbol(
					name.clone(),
					SymbolType::Constant,
					SymbolValue::Number(*value),
					directive.pos.clone(),
					SymbolVisibility::Local,
				)?;
			}
			DirectiveType::Include {
				filename: _,
			} => {
				// Include files would be handled in a pre-processing phase
				// For now, just ignore
			}
			DirectiveType::Bank {
				number,
			} => {
				// Set memory bank - for now, just track it
				self.current_bank = Some(*number as usize);
			}
			DirectiveType::StringEqu {
				name,
				value,
			} => {
				self.symbols.define_symbol(
					name.clone(),
					SymbolType::Constant,
					SymbolValue::String(value.clone()),
					directive.pos.clone(),
					SymbolVisibility::Local,
				)?;
			}
			DirectiveType::IncludeBinary {
				filename: _,
			} => {
				// Include binary files would be handled in a pre-processing phase
				// For now, just ignore
			}
			DirectiveType::List {
				enabled: _,
			} => {
				// List control - for now, just ignore
			}
			DirectiveType::MacroList {
				enabled: _,
			} => {
				// Macro list control - for now, just ignore
			}
			DirectiveType::If {
				condition: _,
			} => {
				// Conditional assembly - for now, just ignore
			}
			DirectiveType::Else => {
				// Conditional else - for now, just ignore
			}
			DirectiveType::EndIf => {
				// End conditional - for now, just ignore
			}
			DirectiveType::IfDef {
				symbol: _,
			} => {
				// Conditional ifdef - for now, just ignore
			}
			DirectiveType::IfNDef {
				symbol: _,
			} => {
				// Conditional ifndef - for now, just ignore
			}
			DirectiveType::InesPrg {
				banks: _,
			}
			| DirectiveType::InesChr {
				banks: _,
			}
			| DirectiveType::InesMap {
				mapper: _,
			}
			| DirectiveType::InesMir {
				mirroring: _,
			}
			| DirectiveType::InesSubMap {
				submapper: _,
			}
			| DirectiveType::InesBat {
				enabled: _,
			}
			| DirectiveType::InesTim {
				timing: _,
			} => {
				// NES-specific directives - for now, just ignore
				// Would be implemented when platform_config is properly defined
			}
			DirectiveType::RsSet {
				value: _,
			}
			| DirectiveType::Rs {
				size: _,
			} => {
				// RS directives - for now, just ignore
			}
			DirectiveType::Fail {
				message,
			} => {
				return Err(AssemblyError::Internal {
					pos: Some(directive.pos.clone()),
					message: message.clone(),
				});
			}
			DirectiveType::MacroStart {
				name: _,
			}
			| DirectiveType::MacroEnd
			| DirectiveType::Function {
				name: _,
				body: _,
			}
			| DirectiveType::ProcStart {
				name: _,
			}
			| DirectiveType::ProcEnd
			| DirectiveType::ProcGroupStart {
				name: _,
			}
			| DirectiveType::ProcGroupEnd => {
				// Macro and procedure directives - for now, just ignore
			}
		}

		Ok(())
	}

	/// Generate final output ROM data
	fn generate_output(&mut self) -> Vec<u8> {
		match self.config.output.format {
			crate::config::OutputFormat::INes => self.generate_ines_rom(),
			crate::config::OutputFormat::Binary => self.generate_binary_rom(),
			_ => {
				// For unsupported formats, return empty vec
				// In a real implementation, this should be handled earlier
				Vec::new()
			}
		}
	}

	/// Generate iNES format ROM
	fn generate_ines_rom(&mut self) -> Vec<u8> {
		let mut rom_data = Vec::new();

		// iNES header (16 bytes)
		rom_data.extend_from_slice(b"NES\x1A"); // Magic number
		rom_data.push(self.config.platform.nes.ines.prg_banks); // PRG ROM size
		rom_data.push(self.config.platform.nes.ines.chr_banks); // CHR ROM size

		// Flags 6
		let mut flags6 = 0u8;
		flags6 |= (self.config.platform.nes.ines.mapper & 0x0F) << 4;
		if self.config.platform.nes.ines.four_screen {
			flags6 |= 0x08;
		}
		if self.config.platform.nes.ines.trainer {
			flags6 |= 0x04;
		}
		if self.config.platform.nes.ines.battery {
			flags6 |= 0x02;
		}
		if self.config.platform.nes.ines.mirroring == crate::config::MirroringType::Vertical {
			flags6 |= 0x01;
		}
		rom_data.push(flags6);

		// Flags 7
		let mut flags7 = 0u8;
		flags7 |= self.config.platform.nes.ines.mapper & 0xF0;
		if self.config.platform.nes.nes2_format {
			flags7 |= 0x08;
		}
		rom_data.push(flags7);

		// Fill remaining header bytes with zeros
		rom_data.extend_from_slice(&[0; 8]);

		// Add PRG ROM data
		for bank in 0..self.config.platform.nes.ines.prg_banks {
			let section_name = format!("PRG{bank}");
			if let Some(section) = self.memory.get_section(&section_name) {
				rom_data.extend_from_slice(&section.data);
			} else {
				// Fill with default value if section doesn't exist
				rom_data
					.resize(rom_data.len() + self.config.platform.nes.memory.prg_bank_size, 0xFF);
			}
		}

		// Add CHR ROM data
		for bank in 0..self.config.platform.nes.ines.chr_banks {
			let section_name = format!("CHR{bank}");
			if let Some(section) = self.memory.get_section(&section_name) {
				rom_data.extend_from_slice(&section.data);
			} else {
				// Fill with zeros if section doesn't exist
				rom_data
					.resize(rom_data.len() + self.config.platform.nes.memory.chr_bank_size, 0x00);
			}
		}

		rom_data
	}

	/// Generate binary format ROM
	fn generate_binary_rom(&mut self) -> Vec<u8> {
		let mut rom_data = Vec::new();

		// Just concatenate all sections in order
		let mut sections: Vec<_> = self.memory.sections().values().collect();
		sections.sort_by_key(|s| s.start_address);

		for section in sections {
			rom_data.extend_from_slice(&section.data);
		}

		rom_data
	}

	/// Get current assembler state
	#[must_use]
	pub fn state(&self) -> AssemblerState {
		self.state.clone()
	}

	/// Get assembly statistics
	#[must_use]
	pub fn stats(&self) -> &AssemblyStats {
		&self.stats
	}

	/// Get memory manager
	#[must_use]
	pub fn memory(&self) -> &MemoryManager {
		&self.memory
	}

	/// Get symbol manager
	#[must_use]
	pub fn symbols(&self) -> &SymbolManager {
		&self.symbols
	}

	/// Get error collector
	#[must_use]
	pub fn errors(&self) -> &ErrorCollector {
		&self.errors
	}

	/// Check if assembly has errors
	#[must_use]
	pub fn has_errors(&self) -> bool {
		self.errors.has_errors()
	}

	/// Reset assembler state for new assembly
	pub fn reset(&mut self) {
		self.state = AssemblerState::Initial;
		self.memory = MemoryManager::new(0xFF);
		self.symbols = SymbolManager::new();
		self.pass_manager = PassManager::new();
		self.source_files.clear();
		self.errors.clear();
		self.stats.reset();
		self.pc = 0;
		self.current_bank = None;
		self.current_section = None;
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::config::ConfigBuilder;

	#[test]
	fn test_assembler_creation() {
		let config = Config::default();
		let assembler = Assembler::new(config);

		assert_eq!(assembler.state(), AssemblerState::Initial);
		assert!(!assembler.has_errors());
		assert_eq!(assembler.stats().files_processed, 0);
	}

	#[test]
	fn test_assembler_reset() {
		let config = Config::default();
		let mut assembler = Assembler::new(config);

		// Modify state
		assembler.state = AssemblerState::Parsing;
		assembler.stats.files_processed = 5;

		// Reset
		assembler.reset();

		assert_eq!(assembler.state(), AssemblerState::Initial);
		assert_eq!(assembler.stats().files_processed, 0);
	}

	#[test]
	fn test_memory_layout_setup() {
		let config = ConfigBuilder::new().mapper(0).prg_banks(2).chr_banks(1).build().unwrap();

		let mut assembler = Assembler::new(config);
		assembler.setup_memory_layout().unwrap();

		// Should have PRG and CHR sections
		assert!(assembler.memory.get_section("PRG_ROM_0").is_some());
		assert!(assembler.memory.get_section("PRG_ROM_1").is_some());
		assert!(assembler.memory.get_section("CHR_ROM_0").is_some());
		assert!(assembler.memory.get_section("ZEROPAGE").is_some());
		assert!(assembler.memory.get_section("RAM").is_some());
	}

	#[test]
	fn test_ines_header_generation() {
		let config = ConfigBuilder::new().mapper(1).prg_banks(2).chr_banks(1).build().unwrap();

		let assembler = Assembler::new(config);
		let mut assembler = assembler;
		let rom = assembler.generate_ines_rom();

		// Check iNES header
		assert_eq!(&rom[0..4], b"NES\x1A");
		assert_eq!(rom[4], 2); // PRG banks
		assert_eq!(rom[5], 1); // CHR banks
		assert_eq!(rom[6] >> 4, 1); // Mapper low nibble
		assert_eq!(rom[7] & 0xF0, 0); // Mapper high nibble
	}
}
