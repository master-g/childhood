//! Assembly pass management
//!
//! This module handles the multi-pass assembly process, coordinating between
//! different assembly phases and managing state transitions.

use crate::error::{AssemblyError, AssemblyResult};

/// Result of a single assembly pass
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassResult {
	/// Pass completed successfully
	Complete,
	/// Pass completed but needs another pass
	NeedsAnotherPass,
	/// Pass completed, continue to next pass
	Continue,
}

/// Type of assembly pass operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassType {
	/// Symbol collection pass
	SymbolCollection,
	/// Code generation pass
	CodeGeneration,
	/// Optimization pass
	Optimization,
	/// Validation pass
	Validation,
}

/// Assembly pass types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pass {
	/// First pass: collect symbols and calculate sizes
	First,
	/// Second pass: generate machine code
	Second,
}

impl Pass {
	/// Get the next pass in sequence
	pub fn next(self) -> Option<Self> {
		match self {
			Self::First => Some(Self::Second),
			Self::Second => None,
		}
	}

	/// Check if this is the first pass
	pub fn is_first(self) -> bool {
		matches!(self, Self::First)
	}

	/// Check if this is the final pass
	pub fn is_final(self) -> bool {
		matches!(self, Self::Second)
	}

	/// Get a human-readable description of this pass
	pub fn description(self) -> &'static str {
		match self {
			Self::First => "Symbol collection and size calculation",
			Self::Second => "Code generation and output",
		}
	}
}

impl std::fmt::Display for Pass {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::First => write!(f, "First Pass"),
			Self::Second => write!(f, "Second Pass"),
		}
	}
}

/// Pass management and coordination
#[derive(Debug)]
pub struct PassManager {
	/// Current assembly pass
	current_pass: Pass,
	/// Maximum number of passes allowed
	max_passes: usize,
	/// Current pass number (1-based)
	pass_number: usize,
	/// Whether to continue on errors during first pass
	continue_on_error: bool,
	/// Statistics for each pass
	pass_stats: Vec<PassStatistics>,
}

/// Statistics collected during a single pass
#[derive(Debug, Clone, Default)]
pub struct PassStatistics {
	/// Pass type
	pub pass: Option<Pass>,
	/// Number of lines processed
	pub lines_processed: usize,
	/// Number of instructions processed
	pub instructions_processed: usize,
	/// Number of symbols defined
	pub symbols_defined: usize,
	/// Number of forward references encountered
	pub forward_references: usize,
	/// Number of errors encountered
	pub errors: usize,
	/// Number of warnings generated
	pub warnings: usize,
	/// Time taken for this pass (in milliseconds)
	pub duration_ms: u64,
}

impl PassManager {
	/// Create a new pass manager
	pub fn new() -> Self {
		Self {
			current_pass: Pass::First,
			max_passes: 2,
			pass_number: 1,
			continue_on_error: false,
			pass_stats: Vec::new(),
		}
	}

	/// Set the current pass
	pub fn set_current_pass(&mut self, pass: Pass) {
		self.current_pass = pass;
		self.pass_number = match pass {
			Pass::First => 1,
			Pass::Second => 2,
		};
	}

	/// Get the current pass
	pub fn current_pass(&self) -> Pass {
		self.current_pass
	}

	/// Get the current pass number (1-based)
	pub fn pass_number(&self) -> usize {
		self.pass_number
	}

	/// Check if this is the first pass
	pub fn is_first_pass(&self) -> bool {
		self.current_pass.is_first()
	}

	/// Check if this is the final pass
	pub fn is_final_pass(&self) -> bool {
		self.current_pass.is_final()
	}

	/// Set maximum number of passes
	pub fn set_max_passes(&mut self, max_passes: usize) {
		self.max_passes = max_passes.max(1);
	}

	/// Set whether to continue on errors during first pass
	pub fn set_continue_on_error(&mut self, continue_on_error: bool) {
		self.continue_on_error = continue_on_error;
	}

	/// Start a new pass and return the previous statistics
	pub fn start_pass(&mut self, pass: Pass) -> Option<PassStatistics> {
		// Save statistics from previous pass if any
		let previous_stats = if !self.pass_stats.is_empty() {
			Some(self.pass_stats.last().unwrap().clone())
		} else {
			None
		};

		// Initialize new pass
		self.set_current_pass(pass);
		self.pass_stats.push(PassStatistics {
			pass: Some(pass),
			..Default::default()
		});

		previous_stats
	}

	/// Complete the current pass with final statistics
	pub fn complete_pass(&mut self, final_stats: PassStatistics) -> AssemblyResult<()> {
		let errors = final_stats.errors;

		if let Some(current_stats) = self.pass_stats.last_mut() {
			*current_stats = final_stats;
			current_stats.pass = Some(self.current_pass);
		}

		// Check if we should continue to next pass
		if self.current_pass.is_final() {
			return Ok(());
		}

		// Check for errors that would prevent continuing
		if !self.continue_on_error && errors > 0 {
			return Err(AssemblyError::FirstPassErrors {
				count: errors,
			});
		}

		Ok(())
	}

	/// Get statistics for a specific pass
	pub fn pass_statistics(&self, pass: Pass) -> Option<&PassStatistics> {
		self.pass_stats.iter().find(|stats| stats.pass == Some(pass))
	}

	/// Get statistics for all completed passes
	pub fn all_statistics(&self) -> &[PassStatistics] {
		&self.pass_stats
	}

	/// Get current pass statistics (mutable)
	pub fn current_statistics_mut(&mut self) -> Option<&mut PassStatistics> {
		self.pass_stats.last_mut()
	}

	/// Check if all required passes are complete
	pub fn is_complete(&self) -> bool {
		self.current_pass.is_final() && !self.pass_stats.is_empty()
	}

	/// Calculate total statistics across all passes
	pub fn total_statistics(&self) -> PassStatistics {
		let mut total = PassStatistics::default();

		for stats in &self.pass_stats {
			total.lines_processed += stats.lines_processed;
			total.instructions_processed += stats.instructions_processed;
			total.symbols_defined += stats.symbols_defined;
			total.forward_references += stats.forward_references;
			total.errors += stats.errors;
			total.warnings += stats.warnings;
			total.duration_ms += stats.duration_ms;
		}

		total
	}

	/// Reset for a new assembly session
	pub fn reset(&mut self) {
		self.current_pass = Pass::First;
		self.pass_number = 1;
		self.pass_stats.clear();
	}

	/// Validate pass configuration
	pub fn validate(&self) -> AssemblyResult<()> {
		if self.max_passes == 0 {
			return Err(AssemblyError::InvalidConfiguration {
				parameter: "max_passes must be at least 1".to_string(),
			});
		}

		if self.max_passes > 10 {
			return Err(AssemblyError::InvalidConfiguration {
				parameter: "max_passes cannot exceed 10".to_string(),
			});
		}

		Ok(())
	}
}

impl Default for PassManager {
	fn default() -> Self {
		Self::new()
	}
}

/// Helper trait for tracking pass-dependent operations
pub trait PassDependent {
	/// Check if this operation should be performed in the current pass
	fn should_execute(&self, pass: Pass) -> bool;

	/// Get the pass priority for this operation (lower = earlier)
	fn pass_priority(&self) -> u8 {
		match self.required_pass() {
			Pass::First => 1,
			Pass::Second => 2,
		}
	}

	/// Get the pass where this operation must be executed
	fn required_pass(&self) -> Pass;
}

/// Pass execution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassStrategy {
	/// Execute in first pass only
	FirstOnly,
	/// Execute in second pass only
	SecondOnly,
	/// Execute in both passes
	Both,
	/// Execute in final pass only
	FinalOnly,
}

impl PassStrategy {
	/// Check if this strategy should execute in the given pass
	pub fn should_execute(self, pass: Pass) -> bool {
		match self {
			Self::FirstOnly => pass.is_first(),
			Self::SecondOnly => matches!(pass, Pass::Second),
			Self::Both => true,
			Self::FinalOnly => pass.is_final(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_pass_sequence() {
		assert_eq!(Pass::First.next(), Some(Pass::Second));
		assert_eq!(Pass::Second.next(), None);
	}

	#[test]
	fn test_pass_properties() {
		assert!(Pass::First.is_first());
		assert!(!Pass::First.is_final());
		assert!(!Pass::Second.is_first());
		assert!(Pass::Second.is_final());
	}

	#[test]
	fn test_pass_display() {
		assert_eq!(Pass::First.to_string(), "First Pass");
		assert_eq!(Pass::Second.to_string(), "Second Pass");
	}

	#[test]
	fn test_pass_manager_creation() {
		let manager = PassManager::new();
		assert_eq!(manager.current_pass(), Pass::First);
		assert_eq!(manager.pass_number(), 1);
		assert!(manager.is_first_pass());
		assert!(!manager.is_final_pass());
	}

	#[test]
	fn test_pass_manager_transitions() {
		let mut manager = PassManager::new();

		// Start first pass
		assert!(manager.start_pass(Pass::First).is_none());
		assert_eq!(manager.current_pass(), Pass::First);

		// Complete first pass
		let first_stats = PassStatistics {
			pass: Some(Pass::First),
			lines_processed: 100,
			errors: 0,
			..Default::default()
		};
		assert!(manager.complete_pass(first_stats).is_ok());

		// Start second pass
		let prev_stats = manager.start_pass(Pass::Second);
		assert!(prev_stats.is_some());
		assert_eq!(manager.current_pass(), Pass::Second);
		assert!(manager.is_final_pass());
	}

	#[test]
	fn test_pass_statistics() {
		let mut manager = PassManager::new();

		manager.start_pass(Pass::First);
		let stats = PassStatistics {
			pass: Some(Pass::First),
			lines_processed: 50,
			symbols_defined: 10,
			errors: 0,
			..Default::default()
		};
		manager.complete_pass(stats.clone()).unwrap();

		let retrieved_stats = manager.pass_statistics(Pass::First);
		assert!(retrieved_stats.is_some());
		assert_eq!(retrieved_stats.unwrap().lines_processed, 50);
		assert_eq!(retrieved_stats.unwrap().symbols_defined, 10);
	}

	#[test]
	fn test_error_handling() {
		let mut manager = PassManager::new();
		manager.set_continue_on_error(false);

		manager.start_pass(Pass::First);
		let stats_with_errors = PassStatistics {
			pass: Some(Pass::First),
			errors: 5,
			..Default::default()
		};

		let result = manager.complete_pass(stats_with_errors);
		assert!(result.is_err());

		if let Err(AssemblyError::FirstPassErrors {
			count,
		}) = result
		{
			assert_eq!(count, 5);
		} else {
			panic!("Expected FirstPassErrors");
		}
	}

	#[test]
	fn test_total_statistics() {
		let mut manager = PassManager::new();

		// First pass
		manager.start_pass(Pass::First);
		manager
			.complete_pass(PassStatistics {
				pass: Some(Pass::First),
				lines_processed: 100,
				symbols_defined: 20,
				duration_ms: 50,
				..Default::default()
			})
			.unwrap();

		// Second pass
		manager.start_pass(Pass::Second);
		manager
			.complete_pass(PassStatistics {
				pass: Some(Pass::Second),
				lines_processed: 100,
				instructions_processed: 80,
				duration_ms: 75,
				..Default::default()
			})
			.unwrap();

		let total = manager.total_statistics();
		assert_eq!(total.lines_processed, 200);
		assert_eq!(total.symbols_defined, 20);
		assert_eq!(total.instructions_processed, 80);
		assert_eq!(total.duration_ms, 125);
	}

	#[test]
	fn test_pass_strategy() {
		assert!(PassStrategy::FirstOnly.should_execute(Pass::First));
		assert!(!PassStrategy::FirstOnly.should_execute(Pass::Second));

		assert!(!PassStrategy::SecondOnly.should_execute(Pass::First));
		assert!(PassStrategy::SecondOnly.should_execute(Pass::Second));

		assert!(PassStrategy::Both.should_execute(Pass::First));
		assert!(PassStrategy::Both.should_execute(Pass::Second));

		assert!(!PassStrategy::FinalOnly.should_execute(Pass::First));
		assert!(PassStrategy::FinalOnly.should_execute(Pass::Second));
	}

	#[test]
	fn test_validation() {
		let mut manager = PassManager::new();
		assert!(manager.validate().is_ok());

		manager.set_max_passes(0);
		assert!(manager.validate().is_err());

		manager.set_max_passes(11);
		assert!(manager.validate().is_err());

		manager.set_max_passes(5);
		assert!(manager.validate().is_ok());
	}
}
