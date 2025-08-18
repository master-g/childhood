//! NES Compiler CLI
//!
//! Command-line interface for the NES assembler and compiler.

use clap::Parser;
use nes_compiler::{Assembler, Config};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(
    name = "nesasm",
    version = nes_compiler::VERSION,
    about = "A modern NES assembler written in Rust",
    long_about = "A modern 6502 assembler with NES support, providing safe and efficient \
                  compilation of assembly code for the Nintendo Entertainment System."
)]
struct Cli {
	/// Input assembly file
	#[arg(value_name = "INPUT")]
	input: PathBuf,

	/// Output ROM file
	#[arg(short, long, value_name = "FILE")]
	output: Option<PathBuf>,

	/// Generate listing file
	#[arg(short = 'i', long)]
	listing: bool,

	/// Listing file path
	#[arg(short = 'L', long, value_name = "FILE")]
	listing_file: Option<PathBuf>,

	/// Macro expansion in listing
	#[arg(short, long)]
	macro_expansion: bool,

	/// Show segment usage
	#[arg(short = 's', long)]
	segment_usage: bool,

	/// Show detailed segment usage
	#[arg(short = 'S', long)]
	detailed_segment_usage: bool,

	/// Listing level (0-3)
	#[arg(short = 'l', long, value_range = 0..=3, default_value = "2")]
	listing_level: u8,

	/// Prevent ROM header generation (raw output)
	#[arg(short, long)]
	raw: bool,

	/// Generate FCEUX symbol files
	#[arg(short = 'f', long, value_name = "PREFIX")]
	symbols: Option<Option<String>>,

	/// Bank offset for FCEUX symbol files
	#[arg(short = 'F', long, value_name = "OFFSET")]
	symbols_offset: Option<u8>,

	/// Enable warnings
	#[arg(short = 'W', long)]
	warnings: bool,

	/// Fill unused ROM space with zeros
	#[arg(short = 'z', long)]
	zero_fill: bool,

	/// Define symbol with integer value (name=value)
	#[arg(short = 'D', long = "equ", value_name = "NAME=VALUE")]
	define: Vec<String>,

	/// Define symbol with string value (name=value)
	#[arg(short = 'C', long = "sequ", value_name = "NAME=VALUE")]
	string_define: Vec<String>,

	/// Verbose output
	#[arg(short, long, action = clap::ArgAction::Count)]
	verbose: u8,

	/// Quiet mode (suppress non-error output)
	#[arg(short, long)]
	quiet: bool,
}

fn main() {
	let cli = Cli::parse();

	// Initialize logging
	init_logging(cli.verbose, cli.quiet);

	// Convert CLI args to Config
	let config = match cli_to_config(&cli) {
		Ok(config) => config,
		Err(e) => {
			eprintln!("Error: {}", e);
			process::exit(1);
		}
	};

	// Create assembler
	let mut assembler = Assembler::new(config);

	// Perform assembly
	match assembler.assemble_file(&cli.input) {
		Ok(rom_data) => {
			if !cli.quiet {
				println!("Assembly successful!");
				println!("ROM size: {} bytes", rom_data.len());

				if cli.segment_usage || cli.detailed_segment_usage {
					print_segment_usage(&assembler, cli.detailed_segment_usage);
				}
			}
		}
		Err(e) => {
			eprintln!("Assembly failed: {}", e);
			process::exit(1);
		}
	}
}

fn init_logging(verbose: u8, quiet: bool) {
	if quiet {
		return;
	}

	let level = match verbose {
		0 => log::LevelFilter::Error,
		1 => log::LevelFilter::Warn,
		2 => log::LevelFilter::Info,
		3 => log::LevelFilter::Debug,
		_ => log::LevelFilter::Trace,
	};

	env_logger::Builder::from_default_env().filter_level(level).init();
}

fn cli_to_config(cli: &Cli) -> Result<Config, Box<dyn std::error::Error>> {
	let mut config = Config::default();

	// Set input and output files
	config.input.source_file = cli.input.clone();
	config.output.rom_file = cli.output.clone().unwrap_or_else(|| {
		let mut output = cli.input.clone();
		output.set_extension("nes");
		output
	});

	// Listing configuration
	if cli.listing {
		config.output.listing_file = cli.listing_file.clone();
	}
	config.assembly.enable_macros = cli.macro_expansion;

	// Output options
	if cli.raw {
		config.output.format = nes_compiler::config::OutputFormat::Binary;
	}
	config.debug.verbose = if cli.quiet {
		0
	} else {
		cli.verbose
	};

	// Assembly options
	if cli.warnings {
		config.assembly.max_errors = u32::MAX;
	}

	// Process symbol definitions
	config.predefined_symbols = HashMap::new();
	config.predefined_strings = HashMap::new();

	for def in &cli.define {
		parse_symbol_definition(def, &mut config, false)?;
	}

	for def in &cli.string_define {
		parse_symbol_definition(def, &mut config, true)?;
	}

	Ok(config)
}

fn parse_symbol_definition(
	def: &str,
	config: &mut Config,
	is_string: bool,
) -> Result<(), Box<dyn std::error::Error>> {
	let parts: Vec<&str> = def.splitn(2, '=').collect();
	if parts.len() != 2 {
		return Err(format!("Invalid symbol definition: {}", def).into());
	}

	let name = parts[0].trim().to_string();
	let value = parts[1].trim();

	if is_string {
		config.predefined_strings.insert(name, value.to_string());
	} else {
		// Parse numeric value (supports $hex, %binary, decimal)
		let numeric_value = if value.starts_with('$') {
			i32::from_str_radix(&value[1..], 16)?
		} else if value.starts_with('%') {
			i32::from_str_radix(&value[1..], 2)?
		} else {
			value.parse::<i32>()?
		};

		config.predefined_symbols.insert(name, numeric_value);
	}

	Ok(())
}

fn print_segment_usage(assembler: &Assembler, detailed: bool) {
	println!("\nSegment Usage:");
	println!("==============");

	// This would be implemented to show memory usage statistics
	// For now, just a placeholder
	if detailed {
		println!("Detailed segment usage reporting not yet implemented");
	} else {
		println!("Basic segment usage reporting not yet implemented");
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse_hex_symbol() {
		let mut config = Config::default();
		parse_symbol_definition("TEST=$FF", &mut config, false).unwrap();
		assert_eq!(config.predefined_symbols.get("TEST"), Some(&255));
	}

	#[test]
	fn test_parse_binary_symbol() {
		let mut config = Config::default();
		parse_symbol_definition("TEST=%11110000", &mut config, false).unwrap();
		assert_eq!(config.predefined_symbols.get("TEST"), Some(&240));
	}

	#[test]
	fn test_parse_decimal_symbol() {
		let mut config = Config::default();
		parse_symbol_definition("TEST=42", &mut config, false).unwrap();
		assert_eq!(config.predefined_symbols.get("TEST"), Some(&42));
	}

	#[test]
	fn test_parse_string_symbol() {
		let mut config = Config::default();
		parse_symbol_definition("MSG=Hello World", &mut config, true).unwrap();
		assert_eq!(config.predefined_strings.get("MSG"), Some(&"Hello World".to_string()));
	}

	#[test]
	fn test_invalid_symbol_definition() {
		let mut config = Config::default();
		assert!(parse_symbol_definition("INVALID", &mut config, false).is_err());
	}
}
