use mynes_common::env::release;

pub fn init() {
	// Initialize tracing and logging
	// Print local CLI version
	println!("{}", release());
}
