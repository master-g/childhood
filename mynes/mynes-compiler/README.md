# NES Compiler

A modern NES assembler and compiler written in Rust, providing a safe and efficient way to compile 6502 assembly code for the Nintendo Entertainment System.

## Overview

NES Compiler is a complete rewrite of traditional NES assemblers, designed with modern software engineering principles. It provides:

- Complete 6502 instruction set support with NES-specific extensions
- Memory-safe implementation using Rust's ownership system
- Comprehensive error reporting with detailed diagnostics
- Multi-pass assembly for forward reference resolution
- Macro system with parameter substitution
- Symbol table management with scoping
- iNES ROM format output with header generation
- Integration with modern development workflows

## Features

### Core Functionality
- **Complete 6502 Support**: Full instruction set including all addressing modes
- **NES-Specific Features**: Built-in support for NES hardware registers and memory layout
- **Multi-Pass Assembly**: Automatic forward reference resolution
- **Memory Safety**: Rust's ownership system prevents common assembly errors

### Advanced Features
- **Macro System**: Powerful macro definitions with parameter substitution
- **Symbol Management**: Comprehensive symbol table with scoping rules
- **Expression Evaluation**: Complex expression parsing with operator precedence
- **Error Recovery**: Continue assembly after errors to catch multiple issues
- **Optimization**: Optional code optimizations for size and performance

### Output Formats
- **iNES ROM**: Standard NES ROM format with proper headers
- **Raw Binary**: Raw machine code output for custom loaders
- **Debug Information**: Symbol tables and debugging information export
- **Listing Files**: Human-readable assembly listings with machine code

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
mynes-compiler = "0.1.0"
```

Or use as a command-line tool:

```bash
cargo install mynes-compiler
```

### Basic Usage

#### Command Line

```bash
# Assemble a simple NES program
nesasm game.asm -o game.nes

# Generate listing file and symbols
nesasm game.asm -o game.nes -i -f

# Show segment usage
nesasm game.asm -s

# Enable warnings and verbose output
nesasm game.asm -W -v
```

#### Library Usage

```rust
use nes_compiler::{Assembler, Config};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new()
        .with_input_file("game.asm".into())
        .with_output_file("game.nes".into())
        .with_warnings()
        .with_listing(None);
    
    let mut assembler = Assembler::new(config);
    let rom_data = assembler.assemble_file(Path::new("game.asm"))?;
    
    println!("Assembly successful! ROM size: {} bytes", rom_data.len());
    Ok(())
}
```

## Assembly Language Syntax

### Basic Instructions

```assembly
; 6502 instructions with various addressing modes
LDA #$42        ; Immediate
LDA $00         ; Zero page
LDA $00,X       ; Zero page,X
LDA $1234       ; Absolute
LDA $1234,X     ; Absolute,X
LDA $1234,Y     ; Absolute,Y
LDA ($00,X)     ; Indexed indirect
LDA ($00),Y     ; Indirect indexed
```

### Labels and Symbols

```assembly
; Global labels
start:
    LDA #$00
    
; Local labels (scoped to nearest global label)
.loop:
    INX
    BNE .loop
    
; Constants
SCREEN_WIDTH = 256
PLAYER_SPEED = 2
```

### Data Definition

```assembly
; Byte data
message:    .db "HELLO WORLD", 0
colors:     .db $0F, $30, $16, $20

; Word data
vectors:    .dw start, nmi_handler, irq_handler

; Reserve space
buffer:     .ds 256         ; Reserve 256 bytes
```

### NES-Specific Features

```assembly
; iNES header directives
.inesprg 2      ; 2 x 16KB PRG ROM banks
.ineschr 1      ; 1 x 8KB CHR ROM bank
.inesmap 1      ; MMC1 mapper
.inesmir 1      ; Vertical mirroring

; Predefined NES registers
LDA PPUSTATUS   ; $2002
STA PPUCTRL     ; $2000
STA PPUADDR     ; $2006
```

### Sections

```assembly
; Zero page variables
.zp
temp:       .rs 1
counter:    .rs 2

; BSS section (uninitialized RAM)
.bss
buffer:     .rs 256

; Code section
.code
.org $8000
start:
    ; Program code here
    
; Data section
.data
lookup_table:   .db $00, $01, $04, $09, $10
```

### Macros

```assembly
; Define a macro
wait_vblank .macro
    bit PPUSTATUS
.loop\@:
    bit PPUSTATUS
    bpl .loop\@
.endm

; Use the macro
    wait_vblank
```

### Expressions and Functions

```assembly
; Arithmetic expressions
LDA #(SCREEN_WIDTH / 8)
STA BASE_ADDR + OFFSET

; Built-in functions
LDA #LOW(start)     ; Low byte of address
LDX #HIGH(start)    ; High byte of address
LDY #BANK(data)     ; Bank number

; User-defined functions
SCREEN_ADDR .func (\1) + ((\2) * 32)
STA SCREEN_ADDR(10, 5)  ; Store at screen position (10,5)
```

## Command Line Options

```
nesasm [OPTIONS] <input.asm>

OPTIONS:
    -o, --output <file>         Output ROM file
    -i, --listing               Generate listing file  
    -L, --listing-file <file>   Specify listing file name
    -m, --macro-expansion       Include macro expansion in listing
    -s, --segment-usage         Show segment usage
    -S, --detailed-usage        Show detailed segment usage
    -l, --listing-level <0-3>   Listing detail level (default: 2)
    -r, --raw                   Generate raw binary (no iNES header)
    -f, --symbols [prefix]      Generate FCEUX symbol files
    -F, --symbols-offset <n>    Bank offset for symbol files
    -W, --warnings              Enable warnings
    -z, --zero-fill             Fill unused ROM with zeros
    -D, --equ <name=value>      Define symbol with integer value
    -C, --sequ <name=value>     Define symbol with string value
    -v, --verbose               Verbose output
    -q, --quiet                 Quiet mode
    -h, --help                  Show help
    -V, --version               Show version
```

## Project Structure

```
nes-compiler/
├── src/
│   ├── core/                   # Core assembler functionality
│   │   ├── assembler.rs        # Main assembler state machine
│   │   ├── machine.rs          # Target machine descriptions
│   │   ├── memory.rs           # Memory management and banking
│   │   └── passes.rs           # Multi-pass coordination
│   ├── parsing/                # Source code parsing
│   │   ├── lexer.rs            # Tokenization
│   │   ├── parser.rs           # Syntax analysis
│   │   ├── expression.rs       # Expression evaluation
│   │   └── directives.rs       # Assembler directives
│   ├── instructions/           # Instruction processing
│   │   ├── cpu6502.rs          # 6502 instruction set
│   │   ├── addressing.rs       # Addressing modes
│   │   └── codegen.rs          # Machine code generation
│   ├── symbols/                # Symbol management
│   │   ├── table.rs            # Symbol table implementation
│   │   ├── scope.rs            # Scoping rules
│   │   └── resolver.rs         # Symbol resolution
│   ├── macros/                 # Macro system
│   │   ├── definition.rs       # Macro definitions
│   │   ├── expansion.rs        # Macro expansion
│   │   └── functions.rs        # User-defined functions
│   ├── output/                 # Output generation
│   │   ├── rom.rs              # ROM file generation
│   │   ├── listing.rs          # Listing file output
│   │   └── symbols.rs          # Symbol export
│   ├── platform/               # Platform-specific code
│   │   ├── nes.rs              # NES-specific features
│   │   └── ines.rs             # iNES format handling
│   ├── config.rs               # Configuration management
│   ├── error.rs                # Error handling
│   ├── utils.rs                # Utility functions
│   ├── lib.rs                  # Library interface
│   └── main.rs                 # CLI application
├── examples/                   # Example programs
├── tests/                      # Integration tests
├── benches/                    # Performance benchmarks
└── README.md                   # This file
```

## Examples

See the [`examples/`](examples/) directory for complete example programs:

- [`hello_world.asm`](examples/hello_world.asm) - Basic "Hello, World!" program
- [`sprite_demo.asm`](examples/sprite_demo.asm) - Sprite movement and animation

## Development Status

**Current Status: Early Development**

This is a modern rewrite of traditional NES assemblers. The project is currently in early development with the following components:

✅ **Completed:**
- Project structure and architecture design
- Core data structures and type definitions
- Error handling framework
- Configuration system
- Memory management system
- Basic CLI interface

🚧 **In Progress:**
- Parser implementation (lexer, syntax analysis)
- 6502 instruction set implementation
- Symbol table and scope management
- Expression evaluation engine

📋 **Planned:**
- Macro system implementation
- Code generation and optimization
- Output format support (iNES, raw binary)
- Comprehensive testing suite
- Documentation and examples
- Performance optimization

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

1. Clone the repository:
```bash
git clone https://github.com/master-g/childhood.git
cd childhood/mynes/nes-compiler
```

2. Build the project:
```bash
cargo build
```

3. Run tests:
```bash
cargo test
```

4. Run examples (once implemented):
```bash
cargo run -- examples/hello_world.asm -o hello_world.nes
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

This project is inspired by and builds upon the work of many NES development tools:

- **NESASM** - The original NES assembler this project reimplements
- **asm6f** - A popular fork of asm6 with many improvements
- **ca65** - Part of the cc65 compiler suite
- **ASM6** - A simple and efficient NES assembler

Special thanks to the NES development community for their continued support and documentation of the platform.

## Related Projects

- [**mynes**](../mynes-bin/) - NES emulator written in Rust
- [**nes_corelib**](../../nes_corelib/) - Core NES functionality library
- [**nesasm**](../../nesasm/) - Original C implementation for reference
