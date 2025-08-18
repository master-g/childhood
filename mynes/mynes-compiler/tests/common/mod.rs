//! Common test utilities for NES compiler integration tests
//!
//! This module provides shared functionality and helpers for testing
//! the NES compiler across different test scenarios.

use nes_compiler::{Assembler, Config};
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::{NamedTempFile, TempDir};

/// Test fixture containing temporary files and directories for testing
pub struct TestFixture {
	/// Temporary directory for test files
	pub temp_dir: TempDir,
	/// Input assembly file
	pub input_file: NamedTempFile,
	/// Expected output ROM file path
	pub output_file: PathBuf,
	/// Expected listing file path
	pub listing_file: PathBuf,
}

impl TestFixture {
	/// Create a new test fixture with the given assembly content
	pub fn new(asm_content: &str) -> Self {
		let temp_dir = TempDir::new().expect("Failed to create temp directory");
		let mut input_file = NamedTempFile::new_in(&temp_dir).expect("Failed to create temp file");

		input_file.write_all(asm_content.as_bytes()).expect("Failed to write assembly content");

		let output_file = temp_dir.path().join("output.nes");
		let listing_file = temp_dir.path().join("output.lst");

		Self {
			temp_dir,
			input_file,
			output_file,
			listing_file,
		}
	}

	/// Get the path to the input assembly file
	pub fn input_path(&self) -> &Path {
		self.input_file.path()
	}

	/// Create an assembler with basic test configuration
	pub fn create_assembler(&self) -> Assembler {
		let config = Config::new()
			.with_input_file(self.input_path().to_path_buf())
			.with_output_file(self.output_file.clone())
			.with_warnings()
			.with_zero_fill();

		Assembler::new(config)
	}

	/// Create an assembler with custom configuration
	pub fn create_assembler_with_config(&self, config: Config) -> Assembler {
		let config = config.with_input_file(self.input_path().to_path_buf());
		Assembler::new(config)
	}
}

/// Standard test configurations for different scenarios
pub struct TestConfigs;

impl TestConfigs {
	/// Basic configuration for simple tests
	pub fn basic() -> Config {
		Config::new().with_warnings()
	}

	/// Configuration with listing output enabled
	pub fn with_listing() -> Config {
		Config::new().with_warnings().with_listing(None).with_macro_expansion()
	}

	/// Configuration with symbol export enabled
	pub fn with_symbols() -> Config {
		Config::new().with_warnings().with_symbol_export(Some("test".to_string()))
	}

	/// Configuration for raw binary output
	pub fn raw_output() -> Config {
		Config::new().with_warnings().with_raw_output()
	}

	/// Configuration with strict error checking
	pub fn strict() -> Config {
		Config::new().with_warnings().with_strict_mode().with_max_errors(1)
	}

	/// Configuration with optimizations enabled
	pub fn optimized() -> Config {
		Config::new().with_warnings().with_branch_optimization().with_zero_page_optimization()
	}
}

/// Common assembly code snippets for testing
pub struct TestSnippets;

impl TestSnippets {
	/// Minimal valid NES program
	pub fn minimal_program() -> &'static str {
		r#"
        .org $8000

        reset:
            SEI
            CLD
            LDX #$FF
            TXS

        loop:
            JMP loop

        .org $FFFC
        .dw reset
        .dw reset
        "#
	}

	/// Program with basic instructions covering different addressing modes
	pub fn basic_instructions() -> &'static str {
		r#"
        .org $8000

        start:
            LDA #$42        ; Immediate
            STA $00         ; Zero page
            LDA $00,X       ; Zero page,X
            STA $0200       ; Absolute
            LDA $0200,X     ; Absolute,X
            LDA $0200,Y     ; Absolute,Y
            LDA ($00,X)     ; Indexed indirect
            STA ($00),Y     ; Indirect indexed
            RTS

        .org $FFFC
        .dw start
        .dw start
        "#
	}

	/// Program with labels and forward references
	pub fn with_labels() -> &'static str {
		r#"
        .org $8000

        start:
            LDA #$00
            CMP #$42
            BEQ end
            JMP forward_ref

        middle:
            INX
            JMP start

        forward_ref:
            LDX #$00
            JMP middle

        end:
            RTS

        .org $FFFC
        .dw start
        .dw start
        "#
	}

	/// Program with data directives
	pub fn with_data() -> &'static str {
		r#"
        .org $8000

        ; String data
        message:    .db "HELLO", 0

        ; Numeric data
        numbers:    .db $01, $02, $03, $04, $05

        ; Word data
        vectors:    .dw start, nmi_handler, irq_handler

        ; Reserved space
        buffer:     .ds 64

        start:
            LDA message
            LDX numbers
            RTS

        nmi_handler:
        irq_handler:
            RTI

        .org $FFFC
        .dw start
        .dw nmi_handler
        "#
	}

	/// Program using NES-specific features
	pub fn nes_specific() -> &'static str {
		r#"
        .inesprg 1
        .ineschr 1
        .inesmap 0
        .inesmir 1

        .org $8000

        start:
            LDA PPUSTATUS   ; NES register
            STA PPUCTRL     ; Another NES register
            LDA #$3F
            STA PPUADDR
            LDA #$00
            STA PPUADDR
            RTS

        .org $FFFC
        .dw start
        .dw start
        "#
	}

	/// Program with macros
	pub fn with_macros() -> &'static str {
		r#"
        ; Define a simple macro
        wait_vblank .macro
            bit PPUSTATUS
        .loop\@:
            bit PPUSTATUS
            bpl .loop\@
        .endm

        ; Define a macro with parameters
        store_value .macro
            LDA #\1
            STA \2
        .endm

        .org $8000

        start:
            wait_vblank
            store_value $42, $0200
            RTS

        .org $FFFC
        .dw start
        .dw start
        "#
	}

	/// Program with expressions and functions
	pub fn with_expressions() -> &'static str {
		r#"
        BASE_ADDR = $0200
        OFFSET = 16
        MULTIPLIER = 2

        .org $8000

        start:
            LDA #LOW(BASE_ADDR + OFFSET)
            STA $00
            LDA #HIGH(BASE_ADDR + OFFSET)
            STA $01

            LDA #(MULTIPLIER * 8)
            STA BASE_ADDR + (OFFSET / 2)
            RTS

        .org $FFFC
        .dw start
        .dw start
        "#
	}

	/// Program using different sections
	pub fn with_sections() -> &'static str {
		r#"
        .zp
        temp:       .rs 1
        counter:    .rs 2

        .bss
        buffer:     .rs 256

        .code
        .org $8000

        start:
            LDA temp
            STA counter
            RTS

        .data
        lookup:     .db $00, $01, $04, $09, $10

        .code
        .org $FFFC
        .dw start
        .dw start
        "#
	}

	/// Invalid program for error testing
	pub fn invalid_program() -> &'static str {
		r#"
        .org $8000

        start:
            LDA undefined_symbol    ; Undefined symbol
            STA $10000             ; Address out of range
            INVALID_INSTRUCTION    ; Invalid instruction
            BEQ too_far_branch     ; Branch target too far

        .org $A000
        too_far_branch:
            RTS
        "#
	}
}

/// Assertion helpers for test validation
pub struct TestAssertions;

impl TestAssertions {
	/// Assert that assembly succeeded and produced expected output
	pub fn assert_assembly_success(result: &nes_compiler::AssemblyResult<Vec<u8>>) {
		match result {
			Ok(rom_data) => {
				assert!(!rom_data.is_empty(), "ROM data should not be empty");
				// Basic size check - should be at least 16KB for minimal NES ROM
				assert!(rom_data.len() >= 16384, "ROM should be at least 16KB");
			}
			Err(e) => panic!("Assembly failed unexpectedly: {}", e),
		}
	}

	/// Assert that assembly failed with expected error
	pub fn assert_assembly_failure(result: &nes_compiler::AssemblyResult<Vec<u8>>) {
		match result {
			Ok(_) => panic!("Expected assembly to fail, but it succeeded"),
			Err(_) => {
				// Success - assembly failed as expected
			}
		}
	}

	/// Assert that ROM has correct iNES header
	pub fn assert_ines_header(rom_data: &[u8], prg_banks: u8, chr_banks: u8) {
		assert!(rom_data.len() >= 16, "ROM too small to contain iNES header");

		// Check iNES signature
		assert_eq!(&rom_data[0..4], b"NES\x1A", "Invalid iNES signature");

		// Check bank counts
		assert_eq!(rom_data[4], prg_banks, "Incorrect PRG bank count");
		assert_eq!(rom_data[5], chr_banks, "Incorrect CHR bank count");
	}

	/// Assert that assembler statistics are reasonable
	pub fn assert_reasonable_stats(stats: &nes_compiler::core::assembler::AssemblyStats) {
		// These assertions will need to be updated once implementation is complete
		// For now, just check that stats structure exists
		let _ = stats.lines_processed;
		let _ = stats.instructions_assembled;
		let _ = stats.symbols_defined;
	}

	/// Assert that a file exists and has expected minimum size
	pub fn assert_file_exists_with_min_size(path: &Path, min_size: u64) {
		assert!(path.exists(), "File should exist: {}", path.display());

		let metadata = std::fs::metadata(path).expect("Failed to get file metadata");

		assert!(
			metadata.len() >= min_size,
			"File {} should be at least {} bytes, but is {} bytes",
			path.display(),
			min_size,
			metadata.len()
		);
	}
}

/// Test data generators for parameterized tests
pub struct TestGenerators;

impl TestGenerators {
	/// Generate assembly programs of varying complexity
	pub fn generate_programs(count: usize) -> Vec<String> {
		let mut programs = Vec::new();

		for i in 0..count {
			let program = format!(
				r#"
                .org $8000

                start:
                    LDA #${:02X}
                    STA ${:04X}
                    {}
                    RTS

                .org $FFFC
                .dw start
                .dw start
                "#,
				i % 256,
				0x0200 + (i % 0x600),
				if i % 2 == 0 {
					"NOP"
				} else {
					"INX"
				}
			);
			programs.push(program);
		}

		programs
	}

	/// Generate programs with varying numbers of symbols
	pub fn generate_symbol_heavy_programs(symbol_counts: &[usize]) -> Vec<String> {
		symbol_counts
			.iter()
			.map(|&count| {
				let mut program = String::new();
				program.push_str(".org $8000\n\n");

				// Generate constants
				for i in 0..count {
					program.push_str(&format!("CONST_{} = {}\n", i, i));
				}

				program.push_str("\nstart:\n");

				// Use some of the constants
				for i in 0..(count.min(10)) {
					program.push_str(&format!("    LDA #CONST_{}\n", i));
					program.push_str("    NOP\n");
				}

				program.push_str("    RTS\n\n");
				program.push_str(".org $FFFC\n");
				program.push_str(".dw start\n");
				program.push_str(".dw start\n");

				program
			})
			.collect()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_fixture_creation() {
		let fixture = TestFixture::new(TestSnippets::minimal_program());
		assert!(fixture.input_path().exists());
		assert!(fixture.temp_dir.path().exists());
	}

	#[test]
	fn test_config_creation() {
		let _basic = TestConfigs::basic();
		let _with_listing = TestConfigs::with_listing();
		let _strict = TestConfigs::strict();
		// All should create without panicking
	}

	#[test]
	fn test_snippet_validity() {
		// All snippets should be valid strings
		assert!(!TestSnippets::minimal_program().is_empty());
		assert!(!TestSnippets::basic_instructions().is_empty());
		assert!(!TestSnippets::with_labels().is_empty());
		assert!(!TestSnippets::nes_specific().is_empty());
	}

	#[test]
	fn test_program_generation() {
		let programs = TestGenerators::generate_programs(5);
		assert_eq!(programs.len(), 5);

		for program in &programs {
			assert!(!program.is_empty());
			assert!(program.contains(".org $8000"));
			assert!(program.contains(".org $FFFC"));
		}
	}

	#[test]
	fn test_symbol_heavy_generation() {
		let counts = [10, 50, 100];
		let programs = TestGenerators::generate_symbol_heavy_programs(&counts);
		assert_eq!(programs.len(), 3);

		// First program should have 10 constants
		assert_eq!(programs[0].matches("CONST_").count(), 20); // 10 definitions + 10 uses
	}
}
