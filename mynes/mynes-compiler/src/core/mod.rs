//! Core assembler functionality.
//!
//! This module contains the main assembler engine and core data structures
//! used throughout the NES compiler.

pub mod assembler;
pub mod machine;
pub mod memory;
pub mod passes;

// Re-exports for convenience
pub use assembler::{Assembler, AssemblerState};
pub use machine::{MachineState, MachineType};
pub use memory::{Bank, MemoryManager, Section, SectionType};
pub use passes::{Pass, PassManager, PassResult, PassType};

/// Default number of assembly passes
pub const DEFAULT_PASSES: usize = 3;

/// Maximum number of assembly passes allowed
pub const MAX_PASSES: usize = 10;

/// Default memory fill value
pub const DEFAULT_FILL_VALUE: u8 = 0xFF;

/// Core assembler result type
pub type CoreResult<T> = crate::error::AssemblyResult<T>;

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_constants() {
		assert_eq!(DEFAULT_PASSES, 3);
		assert_eq!(MAX_PASSES, 10);
		assert_eq!(DEFAULT_FILL_VALUE, 0xFF);
	}
}
