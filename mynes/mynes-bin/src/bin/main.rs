#![deny(clippy::mem_forget)]
#![forbid(unsafe_code)]

use std::future::Future;
use std::process::ExitCode;

use mynes_common::cst;

fn main() -> ExitCode {
	// Initiate the command line
	with_enough_stack(mynes_bin::cli::init())
}

/// Rust's default thread stack size of 2MiB doesn't allow sufficient recursion depth.
fn with_enough_stack<T>(fut: impl Future<Output = T> + Send) -> T {
	// Start a Tokio runtime with custom configuration
	tokio::runtime::Builder::new_multi_thread()
		.enable_all()
		.max_blocking_threads(*cst::RUNTIME_MAX_BLOCKING_THREADS)
		.thread_stack_size(*cst::RUNTIME_STACK_SIZE)
		.thread_name("mynes-worker")
		.build()
		.unwrap()
		.block_on(fut)
}
