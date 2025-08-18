//! Basic assembly integration tests for the NES compiler
//!
//! These tests verify that the compiler can correctly assemble simple
//! NES programs and produce valid output.

use nes_compiler::{Assembler, Config};
use std::io::Write;
use tempfile::NamedTempFile;

/// Helper function to create a temporary assembly file
fn create_temp_asm_file(content: &str) -> NamedTempFile {
	let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
	temp_file.write_all(content.as_bytes()).expect("Failed to write to temp file");
	temp_file
}

/// Helper function to create a basic config for testing
fn create_test_config() -> Config {
	Config::new().with_warnings().with_zero_fill()
}

#[test]
fn test_empty_program() {
	let asm_content = r#"
        ; Empty program - should compile without errors
        .org $8000

        ; Reset vector
        .org $FFFC
        .dw $8000
        .dw $8000
    "#;

	let temp_file = create_temp_asm_file(asm_content);
	let config = create_test_config().with_input_file(temp_file.path().to_path_buf());

	let mut assembler = Assembler::new(config);

	// For now, we expect this to fail since we haven't implemented parsing yet
	// Once parsing is implemented, this should succeed
	let result = assembler.assemble_file(temp_file.path());

	// TODO: Change this to assert!(result.is_ok()) once parsing is implemented
	assert!(result.is_err(), "Expected failure due to unimplemented parsing");
}

#[test]
fn test_simple_instructions() {
	let asm_content = r#"
        .org $8000

        LDA #$42    ; Load immediate
        STA $0200   ; Store to RAM
        LDX #$00    ; Load X register
        LDY #$10    ; Load Y register
        RTS         ; Return

        .org $FFFC
        .dw $8000   ; Reset vector
        .dw $8000   ; IRQ vector
    "#;

	let temp_file = create_temp_asm_file(asm_content);
	let config = create_test_config().with_input_file(temp_file.path().to_path_buf());

	let mut assembler = Assembler::new(config);
	let result = assembler.assemble_file(temp_file.path());

	// TODO: Change this once parsing is implemented
	assert!(result.is_err(), "Expected failure due to unimplemented parsing");
}

#[test]
fn test_labels_and_branches() {
	let asm_content = r#"
        .org $8000

        start:
            LDA #$00
            CMP #$42
            BEQ end
            INC A
            JMP start

        end:
            RTS

        .org $FFFC
        .dw start
        .dw start
    "#;

	let temp_file = create_temp_asm_file(asm_content);
	let config = create_test_config().with_input_file(temp_file.path().to_path_buf());

	let mut assembler = Assembler::new(config);
	let result = assembler.assemble_file(temp_file.path());

	// TODO: Change this once parsing is implemented
	assert!(result.is_err(), "Expected failure due to unimplemented parsing");
}

#[test]
fn test_data_directives() {
	let asm_content = r#"
        .org $8000

        ; Data bytes
        message:    .db "HELLO", 0
        numbers:    .db $01, $02, $03, $04

        ; Data words
        addresses:  .dw $8000, $8010, $8020

        ; Reserve space
        buffer:     .ds 16

        .org $FFFC
        .dw $8000
        .dw $8000
    "#;

	let temp_file = create_temp_asm_file(asm_content);
	let config = create_test_config().with_input_file(temp_file.path().to_path_buf());

	let mut assembler = Assembler::new(config);
	let result = assembler.assemble_file(temp_file.path());

	// TODO: Change this once parsing is implemented
	assert!(result.is_err(), "Expected failure due to unimplemented parsing");
}

#[test]
fn test_nes_specific_directives() {
	let asm_content = r#"
        .inesprg 1
        .ineschr 1
        .inesmap 0
        .inesmir 1

        .org $8000

        LDA PPUSTATUS   ; Use predefined NES register
        STA PPUADDR

        .org $FFFC
        .dw $8000
        .dw $8000
    "#;

	let temp_file = create_temp_asm_file(asm_content);
	let config = create_test_config().with_input_file(temp_file.path().to_path_buf());

	let mut assembler = Assembler::new(config);
	let result = assembler.assemble_file(temp_file.path());

	// TODO: Change this once parsing is implemented
	assert!(result.is_err(), "Expected failure due to unimplemented parsing");
}

#[test]
fn test_expressions() {
	let asm_content = r#"
        VALUE = $42
        OFFSET = 10

        .org $8000

        LDA #VALUE
        STA VALUE + OFFSET
        LDA #LOW(VALUE * 2)
        LDX #HIGH(VALUE * 2)

        .org $FFFC
        .dw $8000
        .dw $8000
    "#;

	let temp_file = create_temp_asm_file(asm_content);
	let config = create_test_config().with_input_file(temp_file.path().to_path_buf());

	let mut assembler = Assembler::new(config);
	let result = assembler.assemble_file(temp_file.path());

	// TODO: Change this once parsing is implemented
	assert!(result.is_err(), "Expected failure due to unimplemented parsing");
}

#[test]
fn test_sections() {
	let asm_content = r#"
        .zp
        temp:       .rs 1
        counter:    .rs 2

        .bss
        buffer:     .rs 256

        .code
        .org $8000

        LDA temp
        STA counter

        .data
        lookup:     .db $00, $01, $02, $03

        .code
        .org $FFFC
        .dw $8000
        .dw $8000
    "#;

	let temp_file = create_temp_asm_file(asm_content);
	let config = create_test_config().with_input_file(temp_file.path().to_path_buf());

	let mut assembler = Assembler::new(config);
	let result = assembler.assemble_file(temp_file.path());

	// TODO: Change this once parsing is implemented
	assert!(result.is_err(), "Expected failure due to unimplemented parsing");
}

#[test]
fn test_error_handling() {
	// Test various error conditions

	// Undefined symbol
	let bad_asm = r#"
        .org $8000
        LDA undefined_symbol
    "#;

	let temp_file = create_temp_asm_file(bad_asm);
	let config = create_test_config().with_input_file(temp_file.path().to_path_buf());

	let mut assembler = Assembler::new(config);
	let result = assembler.assemble_file(temp_file.path());
	assert!(result.is_err(), "Should fail on undefined symbol");
}

#[test]
fn test_predefined_symbols() {
	let asm_content = r#"
        .org $8000

        ; Test predefined NES registers
        LDA PPUCTRL     ; Should be $2000
        STA PPUMASK     ; Should be $2001
        LDA PPUSTATUS   ; Should be $2002

        .org $FFFC
        .dw $8000
        .dw $8000
    "#;

	let temp_file = create_temp_asm_file(asm_content);
	let config = create_test_config()
		.with_input_file(temp_file.path().to_path_buf())
		.with_predefined_symbol("TEST_SYMBOL".to_string(), 42);

	let mut assembler = Assembler::new(config);
	let result = assembler.assemble_file(temp_file.path());

	// TODO: Change this once parsing is implemented
	assert!(result.is_err(), "Expected failure due to unimplemented parsing");
}

#[test]
fn test_configuration_options() {
	let asm_content = r#"
        .org $8000
        RTS
        .org $FFFC
        .dw $8000
        .dw $8000
    "#;

	let temp_file = create_temp_asm_file(asm_content);

	// Test different configuration options
	let configs = vec![
		Config::new().with_raw_output(),
		Config::new().with_zero_fill(),
		Config::new().with_warnings(),
		Config::new().with_strict_mode(),
	];

	for config in configs {
		let config = config.with_input_file(temp_file.path().to_path_buf());
		let mut assembler = Assembler::new(config);
		let result = assembler.assemble_file(temp_file.path());

		// All should fail for now due to unimplemented parsing
		assert!(result.is_err(), "Expected failure due to unimplemented parsing");
	}
}

#[test]
fn test_rom_generation() {
	let asm_content = r#"
        .inesprg 1
        .ineschr 1
        .inesmap 0
        .inesmir 1

        .org $8000

        ; Simple program
        LDA #$42
        STA $0200

        loop:
            JMP loop

        .org $FFFC
        .dw $8000   ; Reset vector
        .dw $8000   ; IRQ vector
    "#;

	let temp_file = create_temp_asm_file(asm_content);
	let config = create_test_config().with_input_file(temp_file.path().to_path_buf());

	let mut assembler = Assembler::new(config);
	let result = assembler.assemble_file(temp_file.path());

	// TODO: Once implemented, check that:
	// - ROM data is generated
	// - ROM size is correct (16KB + 8KB for NROM)
	// - iNES header is present (unless raw output)
	// - Vector table is properly set

	assert!(result.is_err(), "Expected failure due to unimplemented parsing");
}

#[test]
fn test_assembler_statistics() {
	let asm_content = r#"
        .org $8000

        start:
            LDA #$00
            STA $0200
            RTS

        .org $FFFC
        .dw start
        .dw start
    "#;

	let temp_file = create_temp_asm_file(asm_content);
	let config = create_test_config().with_input_file(temp_file.path().to_path_buf());

	let mut assembler = Assembler::new(config);
	let _result = assembler.assemble_file(temp_file.path());

	// Test that statistics are collected
	let stats = assembler.stats();

	// Initially all stats should be 0 since nothing is implemented yet
	assert_eq!(stats.lines_processed, 0);
	assert_eq!(stats.instructions_assembled, 0);
	assert_eq!(stats.symbols_defined, 0);
}
