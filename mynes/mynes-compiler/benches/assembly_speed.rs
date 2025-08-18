//! Assembly speed benchmarks for the NES compiler
//!
//! These benchmarks measure the performance of various assembly operations
//! to help identify bottlenecks and track performance improvements.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use nes_compiler::{Assembler, Config};
use std::io::Write;
use tempfile::NamedTempFile;

/// Create a temporary assembly file with the given content
fn create_temp_asm_file(content: &str) -> NamedTempFile {
	let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
	temp_file.write_all(content.as_bytes()).expect("Failed to write to temp file");
	temp_file
}

/// Generate a simple assembly program with the specified number of instructions
fn generate_simple_program(instruction_count: usize) -> String {
	let mut program = String::new();
	program.push_str(".org $8000\n\n");

	for i in 0..instruction_count {
		program.push_str(&format!("    LDA #${:02X}\n", i % 256));
		program.push_str(&format!("    STA ${:04X}\n", 0x0200 + (i % 0x600)));
	}

	program.push_str("\n.org $FFFC\n");
	program.push_str(".dw $8000\n");
	program.push_str(".dw $8000\n");

	program
}

/// Generate an assembly program with many labels and forward references
fn generate_complex_program(label_count: usize) -> String {
	let mut program = String::new();
	program.push_str(".org $8000\n\n");

	// Generate labels and jumps
	for i in 0..label_count {
		program.push_str(&format!("label_{}:\n", i));
		program.push_str("    LDA #$00\n");
		if i < label_count - 1 {
			program.push_str(&format!("    JMP label_{}\n", i + 1));
		}
	}

	program.push_str("\n.org $FFFC\n");
	program.push_str(".dw label_0\n");
	program.push_str(".dw label_0\n");

	program
}

/// Generate an assembly program with macros
fn generate_macro_program(macro_calls: usize) -> String {
	let mut program = String::new();

	// Define macros
	program.push_str("clear_memory .macro\n");
	program.push_str("    LDA #$00\n");
	program.push_str("    STA \\1\n");
	program.push_str(".endm\n\n");

	program.push_str("increment .macro\n");
	program.push_str("    INC \\1\n");
	program.push_str("    BNE .skip\\@\n");
	program.push_str("    INC \\1+1\n");
	program.push_str(".skip\\@:\n");
	program.push_str(".endm\n\n");

	program.push_str(".org $8000\n\n");

	// Use macros
	for i in 0..macro_calls {
		let addr = 0x0200 + (i * 2) % 0x600;
		program.push_str(&format!("    clear_memory ${:04X}\n", addr));
		program.push_str(&format!("    increment ${:04X}\n", addr));
	}

	program.push_str("\n.org $FFFC\n");
	program.push_str(".dw $8000\n");
	program.push_str(".dw $8000\n");

	program
}

/// Benchmark simple instruction assembly
fn bench_simple_assembly(c: &mut Criterion) {
	let mut group = c.benchmark_group("simple_assembly");

	for instruction_count in [100, 500, 1000, 5000].iter() {
		let program = generate_simple_program(*instruction_count);
		let temp_file = create_temp_asm_file(&program);

		group.throughput(Throughput::Elements(*instruction_count as u64));
		group.bench_with_input(
			BenchmarkId::new("instructions", instruction_count),
			instruction_count,
			|b, _| {
				b.iter(|| {
					let config = Config::new().with_input_file(temp_file.path().to_path_buf());
					let mut assembler = Assembler::new(config);

					// Note: This will fail until parsing is implemented
					// For now, we're benchmarking the setup and error handling
					let _ = assembler.assemble_file(temp_file.path());
				});
			},
		);
	}
	group.finish();
}

/// Benchmark complex programs with many labels
fn bench_complex_assembly(c: &mut Criterion) {
	let mut group = c.benchmark_group("complex_assembly");

	for label_count in [50, 100, 250, 500].iter() {
		let program = generate_complex_program(*label_count);
		let temp_file = create_temp_asm_file(&program);

		group.throughput(Throughput::Elements(*label_count as u64));
		group.bench_with_input(BenchmarkId::new("labels", label_count), label_count, |b, _| {
			b.iter(|| {
				let config = Config::new().with_input_file(temp_file.path().to_path_buf());
				let mut assembler = Assembler::new(config);
				let _ = assembler.assemble_file(temp_file.path());
			});
		});
	}
	group.finish();
}

/// Benchmark macro-heavy programs
fn bench_macro_assembly(c: &mut Criterion) {
	let mut group = c.benchmark_group("macro_assembly");

	for macro_calls in [25, 50, 100, 200].iter() {
		let program = generate_macro_program(*macro_calls);
		let temp_file = create_temp_asm_file(&program);

		group.throughput(Throughput::Elements(*macro_calls as u64));
		group.bench_with_input(
			BenchmarkId::new("macro_calls", macro_calls),
			macro_calls,
			|b, _| {
				b.iter(|| {
					let config = Config::new().with_input_file(temp_file.path().to_path_buf());
					let mut assembler = Assembler::new(config);
					let _ = assembler.assemble_file(temp_file.path());
				});
			},
		);
	}
	group.finish();
}

/// Benchmark assembler creation and initialization
fn bench_assembler_creation(c: &mut Criterion) {
	c.bench_function("assembler_creation", |b| {
		b.iter(|| {
			let config = Config::new();
			let _assembler = Assembler::new(config);
		});
	});
}

/// Benchmark configuration parsing
fn bench_config_creation(c: &mut Criterion) {
	let mut group = c.benchmark_group("config_creation");

	group.bench_function("default", |b| {
		b.iter(|| {
			let _config = Config::default();
		});
	});

	group.bench_function("builder_pattern", |b| {
		b.iter(|| {
			let _config = Config::new()
				.with_warnings()
				.with_zero_fill()
				.with_listing(None)
				.with_symbol_export(Some("game".to_string()))
				.with_max_errors(50);
		});
	});

	group.bench_function("with_predefined_symbols", |b| {
		b.iter(|| {
			let mut config = Config::new();
			for i in 0..100 {
				config = config.with_predefined_symbol(format!("SYMBOL_{}", i), i);
			}
		});
	});

	group.finish();
}

/// Benchmark memory operations (when implemented)
fn bench_memory_operations(c: &mut Criterion) {
	use nes_compiler::core::{Machine, MachineType, MemoryManager};

	let mut group = c.benchmark_group("memory_operations");

	group.bench_function("memory_manager_creation", |b| {
		b.iter(|| {
			let machine = Machine::new(MachineType::Nes);
			let _memory = MemoryManager::new(machine);
		});
	});

	group.bench_function("bank_allocation", |b| {
		let machine = Machine::new(MachineType::Nes);
		let mut memory = MemoryManager::new(machine);

		b.iter(|| {
			for i in 0..10 {
				let _ = memory.select_bank(i);
			}
		});
	});

	group.bench_function("memory_writes", |b| {
		let machine = Machine::new(MachineType::Nes);
		let mut memory = MemoryManager::new(machine);

		b.iter(|| {
			for i in 0..1000u8 {
				let _ = memory.write_byte(i);
			}
		});
	});

	group.finish();
}

/// Benchmark symbol table operations (when implemented)
fn bench_symbol_operations(c: &mut Criterion) {
	use nes_compiler::symbols::SymbolTable;

	let mut group = c.benchmark_group("symbol_operations");

	group.bench_function("symbol_table_creation", |b| {
		b.iter(|| {
			let _symbols = SymbolTable::new();
		});
	});

	// Note: These benchmarks will need to be updated once the symbol table is implemented
	group.bench_function("symbol_definition", |b| {
		let mut symbols = SymbolTable::new();

		b.iter(|| {
			for i in 0..100 {
				// This will fail until implementation is complete
				let _ = symbols.define_constant(format!("SYMBOL_{}", i), i);
			}
		});
	});

	group.finish();
}

/// Benchmark file I/O operations
fn bench_file_operations(c: &mut Criterion) {
	let test_content = generate_simple_program(1000);

	c.bench_function("temp_file_creation", |b| {
		b.iter(|| {
			let _temp_file = create_temp_asm_file(&test_content);
		});
	});
}

criterion_group!(
	benches,
	bench_assembler_creation,
	bench_config_creation,
	bench_memory_operations,
	bench_symbol_operations,
	bench_file_operations,
	bench_simple_assembly,
	bench_complex_assembly,
	bench_macro_assembly
);
criterion_main!(benches);
