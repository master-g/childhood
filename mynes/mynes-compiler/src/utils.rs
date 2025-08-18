//! Utility functions and helpers for the NES compiler
//!
//! This module provides common utility functions, type conversions,
//! and helper functions used throughout the compiler.

use std::fmt;

/// Convert a byte value to hexadecimal string
pub fn byte_to_hex(value: u8) -> String {
	format!("{:02X}", value)
}

/// Convert a word value to hexadecimal string
pub fn word_to_hex(value: u16) -> String {
	format!("{:04X}", value)
}

/// Convert a 32-bit value to hexadecimal string
pub fn dword_to_hex(value: u32) -> String {
	format!("{:08X}", value)
}

/// Parse a numeric literal from assembly source
/// Supports decimal, hexadecimal ($XX), binary (%XX), and character ('X')
pub fn parse_numeric_literal(input: &str) -> Result<i32, String> {
	let trimmed = input.trim();

	if trimmed.is_empty() {
		return Err("Empty numeric literal".to_string());
	}

	// Character literal 'X'
	if trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() == 3 {
		let ch = trimmed.chars().nth(1).unwrap();
		return Ok(ch as u8 as i32);
	}

	// Hexadecimal $XX
	if trimmed.starts_with('$') {
		let hex_part = &trimmed[1..];
		if hex_part.is_empty() {
			return Err("Invalid hexadecimal literal".to_string());
		}
		return i32::from_str_radix(hex_part, 16)
			.map_err(|_| "Invalid hexadecimal literal".to_string());
	}

	// Binary %XX
	if trimmed.starts_with('%') {
		let bin_part = &trimmed[1..];
		if bin_part.is_empty() {
			return Err("Invalid binary literal".to_string());
		}
		return i32::from_str_radix(bin_part, 2).map_err(|_| "Invalid binary literal".to_string());
	}

	// Decimal (default)
	trimmed.parse::<i32>().map_err(|_| "Invalid decimal literal".to_string())
}

/// Check if a value fits in the specified number of bits (unsigned)
pub fn fits_in_bits_unsigned(value: i32, bits: u8) -> bool {
	if value < 0 {
		return false;
	}
	let max_value = (1i64 << bits) - 1;
	(value as i64) <= max_value
}

/// Check if a value fits in the specified number of bits (signed)
pub fn fits_in_bits_signed(value: i32, bits: u8) -> bool {
	let min_value = -(1i64 << (bits - 1));
	let max_value = (1i64 << (bits - 1)) - 1;
	let value_i64 = value as i64;
	value_i64 >= min_value && value_i64 <= max_value
}

/// Calculate relative offset for branch instructions
pub fn calculate_relative_offset(from: u16, to: u16) -> Result<i8, String> {
	// Branch instructions calculate offset from PC + 2
	let adjusted_from = from.wrapping_add(2);
	let offset = to.wrapping_sub(adjusted_from) as i16;

	if offset < -128 || offset > 127 {
		Err(format!("Branch target too far: {} bytes", offset))
	} else {
		Ok(offset as i8)
	}
}

/// Align a value to the specified boundary
pub fn align_to_boundary(value: usize, boundary: usize) -> usize {
	if boundary == 0 || (boundary & (boundary - 1)) != 0 {
		// Not a power of 2
		return value;
	}
	(value + boundary - 1) & !(boundary - 1)
}

/// Get the low byte of a 16-bit value
pub fn low_byte(value: u16) -> u8 {
	(value & 0xFF) as u8
}

/// Get the high byte of a 16-bit value
pub fn high_byte(value: u16) -> u8 {
	((value >> 8) & 0xFF) as u8
}

/// Combine two bytes into a 16-bit word (little-endian)
pub fn make_word(low: u8, high: u8) -> u16 {
	(high as u16) << 8 | (low as u16)
}

/// Check if a string is a valid identifier
pub fn is_valid_identifier(name: &str) -> bool {
	if name.is_empty() {
		return false;
	}

	// First character must be letter, underscore, or dot
	let first_char = name.chars().next().unwrap();
	if !first_char.is_ascii_alphabetic() && first_char != '_' && first_char != '.' {
		return false;
	}

	// Remaining characters must be alphanumeric or underscore
	name.chars().skip(1).all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Escape special characters in a string for display
pub fn escape_string(input: &str) -> String {
	let mut result = String::new();
	for ch in input.chars() {
		match ch {
			'\n' => result.push_str("\\n"),
			'\r' => result.push_str("\\r"),
			'\t' => result.push_str("\\t"),
			'\\' => result.push_str("\\\\"),
			'"' => result.push_str("\\\""),
			'\'' => result.push_str("\\'"),
			c if c.is_control() => result.push_str(&format!("\\x{:02X}", c as u8)),
			c => result.push(c),
		}
	}
	result
}

/// Format a byte slice as a hex dump for debugging
pub fn hex_dump(data: &[u8], bytes_per_line: usize) -> String {
	let mut result = String::new();

	for (offset, chunk) in data.chunks(bytes_per_line).enumerate() {
		// Address
		result.push_str(&format!("{:04X}: ", offset * bytes_per_line));

		// Hex bytes
		for (i, &byte) in chunk.iter().enumerate() {
			if i > 0 && i % 8 == 0 {
				result.push(' ');
			}
			result.push_str(&format!("{:02X} ", byte));
		}

		// Padding for incomplete lines
		let remaining = bytes_per_line - chunk.len();
		for i in 0..remaining {
			if (chunk.len() + i) % 8 == 0 && chunk.len() + i > 0 {
				result.push(' ');
			}
			result.push_str("   ");
		}

		// ASCII representation
		result.push_str(" |");
		for &byte in chunk {
			if byte.is_ascii_graphic() {
				result.push(byte as char);
			} else {
				result.push('.');
			}
		}
		result.push('|');
		result.push('\n');
	}

	result
}

/// Simple CRC-32 calculation for data integrity
pub fn crc32(data: &[u8]) -> u32 {
	const CRC32_TABLE: [u32; 256] = generate_crc32_table();

	let mut crc = 0xFFFFFFFF;
	for &byte in data {
		let table_index = ((crc ^ byte as u32) & 0xFF) as usize;
		crc = (crc >> 8) ^ CRC32_TABLE[table_index];
	}
	!crc
}

/// Generate CRC-32 lookup table at compile time
const fn generate_crc32_table() -> [u32; 256] {
	let mut table = [0u32; 256];
	let mut i = 0;

	while i < 256 {
		let mut crc = i as u32;
		let mut j = 0;

		while j < 8 {
			if crc & 1 != 0 {
				crc = (crc >> 1) ^ 0xEDB88320;
			} else {
				crc >>= 1;
			}
			j += 1;
		}

		table[i] = crc;
		i += 1;
	}

	table
}

/// Format file size in human-readable format
pub fn format_file_size(size: usize) -> String {
	const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
	let mut size_f = size as f64;
	let mut unit_index = 0;

	while size_f >= 1024.0 && unit_index < UNITS.len() - 1 {
		size_f /= 1024.0;
		unit_index += 1;
	}

	if unit_index == 0 {
		format!("{} {}", size, UNITS[unit_index])
	} else {
		format!("{:.1} {}", size_f, UNITS[unit_index])
	}
}

/// Timer for measuring performance
pub struct Timer {
	start: std::time::Instant,
	name: String,
}

impl Timer {
	/// Create a new timer with a name
	pub fn new(name: impl Into<String>) -> Self {
		Self {
			start: std::time::Instant::now(),
			name: name.into(),
		}
	}

	/// Get elapsed time in milliseconds
	pub fn elapsed_ms(&self) -> u64 {
		self.start.elapsed().as_millis() as u64
	}

	/// Get elapsed time in microseconds
	pub fn elapsed_us(&self) -> u64 {
		self.start.elapsed().as_micros() as u64
	}

	/// Reset the timer
	pub fn reset(&mut self) {
		self.start = std::time::Instant::now();
	}
}

impl fmt::Display for Timer {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}: {}ms", self.name, self.elapsed_ms())
	}
}

impl Drop for Timer {
	fn drop(&mut self) {
		if !self.name.is_empty() {
			tracing::debug!("{}", self);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_hex_formatting() {
		assert_eq!(byte_to_hex(0xFF), "FF");
		assert_eq!(byte_to_hex(0x00), "00");
		assert_eq!(word_to_hex(0x1234), "1234");
		assert_eq!(dword_to_hex(0x12345678), "12345678");
	}

	#[test]
	fn test_numeric_literal_parsing() {
		// Decimal
		assert_eq!(parse_numeric_literal("42").unwrap(), 42);
		assert_eq!(parse_numeric_literal("0").unwrap(), 0);

		// Hexadecimal
		assert_eq!(parse_numeric_literal("$FF").unwrap(), 255);
		assert_eq!(parse_numeric_literal("$1234").unwrap(), 0x1234);

		// Binary
		assert_eq!(parse_numeric_literal("%11111111").unwrap(), 255);
		assert_eq!(parse_numeric_literal("%00000000").unwrap(), 0);

		// Character
		assert_eq!(parse_numeric_literal("'A'").unwrap(), 65);
		assert_eq!(parse_numeric_literal("'0'").unwrap(), 48);

		// Invalid cases
		assert!(parse_numeric_literal("").is_err());
		assert!(parse_numeric_literal("$").is_err());
		assert!(parse_numeric_literal("%").is_err());
		assert!(parse_numeric_literal("invalid").is_err());
	}

	#[test]
	fn test_bit_fitting() {
		// Unsigned
		assert!(fits_in_bits_unsigned(255, 8));
		assert!(!fits_in_bits_unsigned(256, 8));
		assert!(!fits_in_bits_unsigned(-1, 8));

		// Signed
		assert!(fits_in_bits_signed(127, 8));
		assert!(fits_in_bits_signed(-128, 8));
		assert!(!fits_in_bits_signed(128, 8));
		assert!(!fits_in_bits_signed(-129, 8));
	}

	#[test]
	fn test_relative_offset() {
		// Forward branch
		assert_eq!(calculate_relative_offset(0x8000, 0x8010).unwrap(), 14);

		// Backward branch
		assert_eq!(calculate_relative_offset(0x8010, 0x8000).unwrap(), -18);

		// Too far
		assert!(calculate_relative_offset(0x8000, 0x8100).is_err());
	}

	#[test]
	fn test_alignment() {
		assert_eq!(align_to_boundary(10, 4), 12);
		assert_eq!(align_to_boundary(16, 4), 16);
		assert_eq!(align_to_boundary(0, 4), 0);
		assert_eq!(align_to_boundary(10, 3), 10); // Not power of 2
	}

	#[test]
	fn test_byte_operations() {
		assert_eq!(low_byte(0x1234), 0x34);
		assert_eq!(high_byte(0x1234), 0x12);
		assert_eq!(make_word(0x34, 0x12), 0x1234);
	}

	#[test]
	fn test_identifier_validation() {
		assert!(is_valid_identifier("label"));
		assert!(is_valid_identifier("_start"));
		assert!(is_valid_identifier(".local"));
		assert!(is_valid_identifier("test123"));

		assert!(!is_valid_identifier(""));
		assert!(!is_valid_identifier("123label"));
		assert!(!is_valid_identifier("label-name"));
		assert!(!is_valid_identifier("label with spaces"));
	}

	#[test]
	fn test_string_escaping() {
		assert_eq!(escape_string("hello"), "hello");
		assert_eq!(escape_string("hello\nworld"), "hello\\nworld");
		assert_eq!(escape_string("quote\"test"), "quote\\\"test");
	}

	#[test]
	fn test_hex_dump() {
		let data = [0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE, 0xFD, 0xFC];
		let dump = hex_dump(&data, 8);
		assert!(dump.contains("0000:"));
		assert!(dump.contains("00 01 02 03"));
		assert!(dump.contains("FF FE FD FC"));
	}

	#[test]
	fn test_crc32() {
		let data = b"hello world";
		let crc = crc32(data);
		// Verify it returns consistent results
		assert_eq!(crc, crc32(data));

		// Different data should produce different CRC
		let different_data = b"hello world!";
		assert_ne!(crc, crc32(different_data));
	}

	#[test]
	fn test_file_size_formatting() {
		assert_eq!(format_file_size(500), "500 B");
		assert_eq!(format_file_size(1024), "1.0 KB");
		assert_eq!(format_file_size(1536), "1.5 KB");
		assert_eq!(format_file_size(1024 * 1024), "1.0 MB");
	}

	#[test]
	fn test_timer() {
		let timer = Timer::new("test");
		std::thread::sleep(std::time::Duration::from_millis(1));
		assert!(timer.elapsed_ms() >= 1);
		assert!(timer.elapsed_us() >= 1000);
	}
}
