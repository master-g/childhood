//! Memory management for the NES assembler.
//!
//! This module provides memory management functionality including bank management,
//! section handling, and address space organization for NES development.

use std::collections::HashMap;
use std::fmt;

use crate::error::{AssemblyError, AssemblyResult, SourcePos};

/// Default bank size for NES (8KB for CHR, 16KB for PRG)
pub const DEFAULT_BANK_SIZE: usize = 8192;

/// Maximum address space for 6502
pub const MAX_ADDRESS: u16 = 0xFFFF;

/// Zero page size
pub const ZERO_PAGE_SIZE: u16 = 0x100;

/// Memory section types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SectionType {
	/// Program code section
	Code,
	/// Data section
	Data,
	/// Read-only data section
	RoData,
	/// Uninitialized data section
	Bss,
	/// Zero page section
	ZeroPage,
	/// CHR ROM data
	ChrRom,
	/// Custom section
	Custom(&'static str),
}

impl fmt::Display for SectionType {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Code => write!(f, "CODE"),
			Self::Data => write!(f, "DATA"),
			Self::RoData => write!(f, "RODATA"),
			Self::Bss => write!(f, "BSS"),
			Self::ZeroPage => write!(f, "ZEROPAGE"),
			Self::ChrRom => write!(f, "CHRROM"),
			Self::Custom(name) => write!(f, "{}", name),
		}
	}
}

/// Memory section definition
#[derive(Debug, Clone)]
pub struct Section {
	/// Section name
	pub name: String,
	/// Section type
	pub section_type: SectionType,
	/// Start address
	pub start_address: u16,
	/// Current write position
	pub current_address: u16,
	/// End address (exclusive)
	pub end_address: u16,
	/// Section data
	pub data: Vec<u8>,
	/// Whether section can be written to
	pub writable: bool,
	/// Fill value for uninitialized areas
	pub fill_value: u8,
	/// Bank number this section belongs to
	pub bank: Option<usize>,
}

impl Section {
	/// Create a new memory section
	pub fn new(
		name: String,
		section_type: SectionType,
		start_address: u16,
		size: u16,
		writable: bool,
		fill_value: u8,
	) -> Self {
		let end_address = start_address.saturating_add(size);
		let data = vec![fill_value; size as usize];

		Self {
			name,
			section_type,
			start_address,
			current_address: start_address,
			end_address,
			data,
			writable,
			fill_value,
			bank: None,
		}
	}

	/// Get the size of the section
	pub fn size(&self) -> u16 {
		self.end_address - self.start_address
	}

	/// Get the remaining size in the section
	pub fn remaining_size(&self) -> u16 {
		self.end_address - self.current_address
	}

	/// Check if an address is within this section
	pub fn contains_address(&self, address: u16) -> bool {
		address >= self.start_address && address < self.end_address
	}

	/// Write data to the section at current position
	pub fn write_data(&mut self, data: &[u8], pos: &SourcePos) -> AssemblyResult<()> {
		if !self.writable {
			return Err(AssemblyError::memory(
				pos.clone(),
				format!("Section '{}' is read-only", self.name),
			));
		}

		if self.current_address.saturating_add(data.len() as u16) > self.end_address {
			return Err(AssemblyError::bank_overflow(
				pos.clone(),
				data.len(),
				self.remaining_size() as usize,
			));
		}

		let offset = (self.current_address - self.start_address) as usize;
		self.data[offset..offset + data.len()].copy_from_slice(data);
		self.current_address += data.len() as u16;

		Ok(())
	}

	/// Write data to a specific address in the section
	pub fn write_at_address(
		&mut self,
		address: u16,
		data: &[u8],
		pos: &SourcePos,
	) -> AssemblyResult<()> {
		if !self.writable {
			return Err(AssemblyError::memory(
				pos.clone(),
				format!("Section '{}' is read-only", self.name),
			));
		}

		if !self.contains_address(address) {
			return Err(AssemblyError::invalid_address(
				pos.clone(),
				address,
				format!("Address not in section '{}'", self.name),
			));
		}

		if address.saturating_add(data.len() as u16) > self.end_address {
			return Err(AssemblyError::bank_overflow(
				pos.clone(),
				data.len(),
				(self.end_address - address) as usize,
			));
		}

		let offset = (address - self.start_address) as usize;
		self.data[offset..offset + data.len()].copy_from_slice(data);

		Ok(())
	}

	/// Read data from the section
	pub fn read_data(&self, address: u16, len: usize) -> Option<&[u8]> {
		if !self.contains_address(address) {
			return None;
		}

		let offset = (address - self.start_address) as usize;
		let end_offset = offset + len;

		if end_offset <= self.data.len() {
			Some(&self.data[offset..end_offset])
		} else {
			None
		}
	}

	/// Set the current write position
	pub fn set_position(&mut self, address: u16, pos: &SourcePos) -> AssemblyResult<()> {
		if !self.contains_address(address) {
			return Err(AssemblyError::invalid_address(
				pos.clone(),
				address,
				format!("Address not in section '{}'", self.name),
			));
		}

		self.current_address = address;
		Ok(())
	}

	/// Advance the current position
	pub fn advance(&mut self, bytes: u16, pos: &SourcePos) -> AssemblyResult<()> {
		let new_address = self.current_address.saturating_add(bytes);
		if new_address > self.end_address {
			return Err(AssemblyError::bank_overflow(
				pos.clone(),
				bytes as usize,
				self.remaining_size() as usize,
			));
		}

		self.current_address = new_address;
		Ok(())
	}

	/// Get the current position
	pub fn current_position(&self) -> u16 {
		self.current_address
	}

	/// Reset the section to its initial state
	pub fn reset(&mut self) {
		self.current_address = self.start_address;
		self.data.fill(self.fill_value);
	}

	/// Check if the section is empty (no data written)
	pub fn is_empty(&self) -> bool {
		self.current_address == self.start_address
	}

	/// Get usage statistics
	pub fn usage(&self) -> SectionUsage {
		let used_bytes = (self.current_address - self.start_address) as usize;
		let total_bytes = self.size() as usize;
		let usage_percent = if total_bytes > 0 {
			(used_bytes * 100) / total_bytes
		} else {
			0
		};

		SectionUsage {
			used_bytes,
			total_bytes,
			usage_percent,
		}
	}
}

/// Section usage statistics
#[derive(Debug, Clone)]
pub struct SectionUsage {
	pub used_bytes: usize,
	pub total_bytes: usize,
	pub usage_percent: usize,
}

impl fmt::Display for SectionUsage {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}/{} bytes ({}%)", self.used_bytes, self.total_bytes, self.usage_percent)
	}
}

/// Memory bank representation
#[derive(Debug, Clone)]
pub struct Bank {
	/// Bank number
	pub number: usize,
	/// Bank size in bytes
	pub size: usize,
	/// Sections in this bank
	pub sections: Vec<String>,
	/// Bank type
	pub bank_type: BankType,
	/// Whether the bank is active
	pub active: bool,
}

impl Bank {
	/// Create a new memory bank
	pub fn new(number: usize, size: usize, bank_type: BankType) -> Self {
		Self {
			number,
			size,
			sections: Vec::new(),
			bank_type,
			active: false,
		}
	}

	/// Add a section to this bank
	pub fn add_section(&mut self, section_name: String) {
		if !self.sections.contains(&section_name) {
			self.sections.push(section_name);
		}
	}

	/// Remove a section from this bank
	pub fn remove_section(&mut self, section_name: &str) {
		self.sections.retain(|s| s != section_name);
	}

	/// Check if the bank contains a section
	pub fn contains_section(&self, section_name: &str) -> bool {
		self.sections.contains(&section_name.to_string())
	}
}

/// Bank type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BankType {
	/// Program ROM bank
	PrgRom,
	/// Character ROM bank
	ChrRom,
	/// Work RAM bank
	WorkRam,
	/// Save RAM bank
	SaveRam,
}

impl fmt::Display for BankType {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::PrgRom => write!(f, "PRG-ROM"),
			Self::ChrRom => write!(f, "CHR-ROM"),
			Self::WorkRam => write!(f, "WRAM"),
			Self::SaveRam => write!(f, "SRAM"),
		}
	}
}

/// Memory manager for organizing sections and banks
#[derive(Debug)]
pub struct MemoryManager {
	/// All memory sections
	sections: HashMap<String, Section>,
	/// All memory banks
	banks: HashMap<usize, Bank>,
	/// Current active section
	current_section: Option<String>,
	/// Current active bank
	current_bank: Option<usize>,
	/// Default fill value
	fill_value: u8,
}

impl MemoryManager {
	/// Create a new memory manager
	pub fn new(fill_value: u8) -> Self {
		Self {
			sections: HashMap::new(),
			banks: HashMap::new(),
			current_section: None,
			current_bank: None,
			fill_value,
		}
	}

	/// Create a new section
	pub fn create_section(
		&mut self,
		name: String,
		section_type: SectionType,
		start_address: u16,
		size: u16,
		writable: bool,
		pos: &SourcePos,
	) -> AssemblyResult<()> {
		if self.sections.contains_key(&name) {
			return Err(AssemblyError::memory(
				pos.clone(),
				format!("Section '{}' already exists", name),
			));
		}

		// Check for address conflicts
		let end_address = start_address.saturating_add(size);
		for (existing_name, existing_section) in &self.sections {
			if !(end_address <= existing_section.start_address
				|| start_address >= existing_section.end_address)
			{
				return Err(AssemblyError::memory(
					pos.clone(),
					format!(
						"Section '{}' conflicts with existing section '{}'",
						name, existing_name
					),
				));
			}
		}

		let section = Section::new(
			name.clone(),
			section_type,
			start_address,
			size,
			writable,
			self.fill_value,
		);
		self.sections.insert(name, section);

		Ok(())
	}

	/// Get a section by name
	pub fn get_section(&self, name: &str) -> Option<&Section> {
		self.sections.get(name)
	}

	/// Get a mutable section by name
	pub fn get_section_mut(&mut self, name: &str) -> Option<&mut Section> {
		self.sections.get_mut(name)
	}

	/// Set the current active section
	pub fn set_current_section(&mut self, name: &str, pos: &SourcePos) -> AssemblyResult<()> {
		if !self.sections.contains_key(name) {
			return Err(AssemblyError::memory(
				pos.clone(),
				format!("Section '{}' does not exist", name),
			));
		}

		self.current_section = Some(name.to_string());
		Ok(())
	}

	/// Get the current active section
	pub fn current_section(&self) -> Option<&Section> {
		self.current_section.as_ref().and_then(|name| self.sections.get(name))
	}

	/// Get the current active section mutably
	pub fn current_section_mut(&mut self) -> Option<&mut Section> {
		let name = self.current_section.clone()?;
		self.sections.get_mut(&name)
	}

	/// Write data to the current section
	pub fn write_data(&mut self, data: &[u8], pos: &SourcePos) -> AssemblyResult<()> {
		if let Some(section) = self.current_section_mut() {
			section.write_data(data, pos)
		} else {
			Err(AssemblyError::memory(pos.clone(), "No active section for writing".to_string()))
		}
	}

	/// Write data to a specific address
	pub fn write_at_address(
		&mut self,
		address: u16,
		data: &[u8],
		pos: &SourcePos,
	) -> AssemblyResult<()> {
		// Find the section containing this address
		let section_name = self
			.sections
			.iter()
			.find(|(_, section)| section.contains_address(address))
			.map(|(name, _)| name.clone());

		if let Some(name) = section_name {
			if let Some(section) = self.sections.get_mut(&name) {
				section.write_at_address(address, data, pos)
			} else {
				Err(AssemblyError::memory(pos.clone(), format!("Section '{}' not found", name)))
			}
		} else {
			Err(AssemblyError::invalid_address(
				pos.clone(),
				address,
				"Address not in any section".to_string(),
			))
		}
	}

	/// Read data from any section
	pub fn read_data(&self, address: u16, len: usize) -> Option<&[u8]> {
		for section in self.sections.values() {
			if let Some(data) = section.read_data(address, len) {
				return Some(data);
			}
		}
		None
	}

	/// Create a new bank
	pub fn create_bank(
		&mut self,
		number: usize,
		size: usize,
		bank_type: BankType,
		pos: &SourcePos,
	) -> AssemblyResult<()> {
		if self.banks.contains_key(&number) {
			return Err(AssemblyError::memory(
				pos.clone(),
				format!("Bank {} already exists", number),
			));
		}

		let bank = Bank::new(number, size, bank_type);
		self.banks.insert(number, bank);

		Ok(())
	}

	/// Get a bank by number
	pub fn get_bank(&self, number: usize) -> Option<&Bank> {
		self.banks.get(&number)
	}

	/// Get a mutable bank by number
	pub fn get_bank_mut(&mut self, number: usize) -> Option<&mut Bank> {
		self.banks.get_mut(&number)
	}

	/// Set the current active bank
	pub fn set_current_bank(&mut self, number: usize, pos: &SourcePos) -> AssemblyResult<()> {
		if !self.banks.contains_key(&number) {
			return Err(AssemblyError::memory(
				pos.clone(),
				format!("Bank {} does not exist", number),
			));
		}

		// Deactivate current bank
		if let Some(current) = self.current_bank {
			if let Some(bank) = self.banks.get_mut(&current) {
				bank.active = false;
			}
		}

		// Activate new bank
		if let Some(bank) = self.banks.get_mut(&number) {
			bank.active = true;
		}

		self.current_bank = Some(number);
		Ok(())
	}

	/// Get the current active bank
	pub fn current_bank(&self) -> Option<&Bank> {
		self.current_bank.and_then(|number| self.banks.get(&number))
	}

	/// Assign a section to a bank
	pub fn assign_section_to_bank(
		&mut self,
		section_name: &str,
		bank_number: usize,
		pos: &SourcePos,
	) -> AssemblyResult<()> {
		if !self.sections.contains_key(section_name) {
			return Err(AssemblyError::memory(
				pos.clone(),
				format!("Section '{}' does not exist", section_name),
			));
		}

		if !self.banks.contains_key(&bank_number) {
			return Err(AssemblyError::memory(
				pos.clone(),
				format!("Bank {} does not exist", bank_number),
			));
		}

		// Update section
		if let Some(section) = self.sections.get_mut(section_name) {
			section.bank = Some(bank_number);
		}

		// Update bank
		if let Some(bank) = self.banks.get_mut(&bank_number) {
			bank.add_section(section_name.to_string());
		}

		Ok(())
	}

	/// Get all sections
	pub fn sections(&self) -> &HashMap<String, Section> {
		&self.sections
	}

	/// Get all banks
	pub fn banks(&self) -> &HashMap<usize, Bank> {
		&self.banks
	}

	/// Get memory usage statistics
	pub fn usage_stats(&self) -> MemoryUsage {
		let mut total_used = 0;
		let mut total_available = 0;
		let mut section_stats = HashMap::new();

		for (name, section) in &self.sections {
			let usage = section.usage();
			total_used += usage.used_bytes;
			total_available += usage.total_bytes;
			section_stats.insert(name.clone(), usage);
		}

		MemoryUsage {
			total_used,
			total_available,
			section_stats,
		}
	}

	/// Reset all sections
	pub fn reset(&mut self) {
		for section in self.sections.values_mut() {
			section.reset();
		}
		self.current_section = None;
		self.current_bank = None;
	}

	/// Generate a memory map
	pub fn memory_map(&self) -> Vec<MemoryMapEntry> {
		let mut entries: Vec<MemoryMapEntry> = self
			.sections
			.values()
			.map(|section| MemoryMapEntry {
				name: section.name.clone(),
				section_type: section.section_type,
				start_address: section.start_address,
				end_address: section.end_address,
				size: section.size(),
				bank: section.bank,
				usage: section.usage(),
			})
			.collect();

		entries.sort_by_key(|entry| entry.start_address);
		entries
	}
}

/// Memory usage statistics
#[derive(Debug)]
pub struct MemoryUsage {
	pub total_used: usize,
	pub total_available: usize,
	pub section_stats: HashMap<String, SectionUsage>,
}

impl fmt::Display for MemoryUsage {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let usage_percent = if self.total_available > 0 {
			(self.total_used * 100) / self.total_available
		} else {
			0
		};

		writeln!(
			f,
			"Total Memory Usage: {}/{} bytes ({}%)",
			self.total_used, self.total_available, usage_percent
		)?;

		for (name, stats) in &self.section_stats {
			writeln!(f, "  {}: {}", name, stats)?;
		}

		Ok(())
	}
}

/// Memory map entry
#[derive(Debug, Clone)]
pub struct MemoryMapEntry {
	pub name: String,
	pub section_type: SectionType,
	pub start_address: u16,
	pub end_address: u16,
	pub size: u16,
	pub bank: Option<usize>,
	pub usage: SectionUsage,
}

impl fmt::Display for MemoryMapEntry {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let bank_str = if let Some(bank) = self.bank {
			format!(" (Bank {})", bank)
		} else {
			String::new()
		};

		write!(
			f,
			"{:12} {:8} ${:04X}-${:04X} {:6} bytes{} - {}",
			self.name,
			self.section_type,
			self.start_address,
			self.end_address - 1,
			self.size,
			bank_str,
			self.usage
		)
	}
}

impl Default for MemoryManager {
	fn default() -> Self {
		Self::new(crate::core::DEFAULT_FILL_VALUE)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::path::PathBuf;

	fn test_pos() -> SourcePos {
		SourcePos::new(PathBuf::from("test.asm"), 1, 1)
	}

	#[test]
	fn test_section_creation() {
		let section =
			Section::new("TEST".to_string(), SectionType::Code, 0x8000, 0x1000, true, 0xFF);

		assert_eq!(section.name, "TEST");
		assert_eq!(section.section_type, SectionType::Code);
		assert_eq!(section.start_address, 0x8000);
		assert_eq!(section.end_address, 0x9000);
		assert_eq!(section.size(), 0x1000);
		assert!(section.writable);
	}

	#[test]
	fn test_section_write() {
		let mut section =
			Section::new("TEST".to_string(), SectionType::Code, 0x8000, 0x100, true, 0xFF);

		let data = vec![0x01, 0x02, 0x03];
		let pos = test_pos();

		assert!(section.write_data(&data, &pos).is_ok());
		assert_eq!(section.current_address, 0x8003);

		let read_data = section.read_data(0x8000, 3).unwrap();
		assert_eq!(read_data, &[0x01, 0x02, 0x03]);
	}

	#[test]
	fn test_section_overflow() {
		let mut section =
			Section::new("TEST".to_string(), SectionType::Code, 0x8000, 0x02, true, 0xFF);

		let data = vec![0x01, 0x02, 0x03]; // Too big for section
		let pos = test_pos();

		assert!(section.write_data(&data, &pos).is_err());
	}

	#[test]
	fn test_memory_manager() {
		let mut mm = MemoryManager::new(0xFF);
		let pos = test_pos();

		// Create a section
		assert!(
			mm.create_section("CODE".to_string(), SectionType::Code, 0x8000, 0x1000, true, &pos)
				.is_ok()
		);

		// Set current section
		assert!(mm.set_current_section("CODE", &pos).is_ok());

		// Write data
		let data = vec![0xEA, 0xEA, 0xEA]; // NOP instructions
		assert!(mm.write_data(&data, &pos).is_ok());

		// Verify data
		let read_data = mm.read_data(0x8000, 3).unwrap();
		assert_eq!(read_data, &[0xEA, 0xEA, 0xEA]);
	}

	#[test]
	fn test_bank_management() {
		let mut mm = MemoryManager::new(0xFF);
		let pos = test_pos();

		// Create a bank
		assert!(mm.create_bank(0, 16384, BankType::PrgRom, &pos).is_ok());

		// Create a section
		assert!(
			mm.create_section("CODE".to_string(), SectionType::Code, 0x8000, 0x1000, true, &pos)
				.is_ok()
		);

		// Assign section to bank
		assert!(mm.assign_section_to_bank("CODE", 0, &pos).is_ok());

		// Verify assignment
		let section = mm.get_section("CODE").unwrap();
		assert_eq!(section.bank, Some(0));

		let bank = mm.get_bank(0).unwrap();
		assert!(bank.contains_section("CODE"));
	}

	#[test]
	fn test_memory_usage() {
		let mut mm = MemoryManager::new(0xFF);
		let pos = test_pos();

		// Create sections
		assert!(
			mm.create_section("CODE".to_string(), SectionType::Code, 0x8000, 0x1000, true, &pos)
				.is_ok()
		);

		assert!(mm.set_current_section("CODE", &pos).is_ok());

		let data = vec![0xEA; 100]; // 100 bytes
		assert!(mm.write_data(&data, &pos).is_ok());

		let usage = mm.usage_stats();
		assert_eq!(usage.total_used, 100);
		assert_eq!(usage.total_available, 0x1000);
	}

	#[test]
	fn test_memory_map() {
		let mut mm = MemoryManager::new(0xFF);
		let pos = test_pos();

		// Create multiple sections
		assert!(
			mm.create_section("CODE".to_string(), SectionType::Code, 0x8000, 0x1000, true, &pos)
				.is_ok()
		);

		assert!(
			mm.create_section("DATA".to_string(), SectionType::Data, 0x6000, 0x800, true, &pos)
				.is_ok()
		);

		let memory_map = mm.memory_map();
		assert_eq!(memory_map.len(), 2);

		// Should be sorted by address
		assert_eq!(memory_map[0].start_address, 0x6000);
		assert_eq!(memory_map[1].start_address, 0x8000);
	}
}
