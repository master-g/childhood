use once_cell::sync::Lazy;

pub const LOGO: &str = r"
░  ░░░░  ░░  ░░░░  ░░   ░░░  ░░        ░░░      ░░
▒   ▒▒   ▒▒▒  ▒▒  ▒▒▒    ▒▒  ▒▒  ▒▒▒▒▒▒▒▒  ▒▒▒▒▒▒▒
▓        ▓▓▓▓    ▓▓▓▓  ▓  ▓  ▓▓      ▓▓▓▓▓      ▓▓
█  █  █  █████  █████  ██    ██  ██████████████  █
█  ████  █████  █████  ███   ██        ███      ██
";

/// The publicly visible name of the server
#[allow(dead_code)]
pub const PKG_NAME: &str = "mynes";

/// The publicly visible user-agent of the command-line tool
#[allow(dead_code)]
pub const SERVER_AGENT: &str = concat!("mynes ", env!("CARGO_PKG_VERSION"));

/// What is the runtime thread memory stack size (defaults to 10MiB)
pub static RUNTIME_STACK_SIZE: Lazy<usize> = Lazy::new(|| {
	// Stack frames are generally larger in debug mode.
	let default = if cfg!(debug_assertions) {
		20 * 1024 * 1024 // 20MiB in debug mode
	} else {
		10 * 1024 * 1024 // 10MiB in release mode
	};
	option_env!("MYNES_RUNTIME_STACK_SIZE").and_then(|s| s.parse::<usize>().ok()).unwrap_or(default)
});

/// How many threads which can be started for blocking operations (defaults to 512)
pub static RUNTIME_MAX_BLOCKING_THREADS: Lazy<usize> = Lazy::new(|| {
	option_env!("MYNES_RUNTIME_MAX_BLOCKING_THREADS")
		.and_then(|s| s.parse::<usize>().ok())
		.unwrap_or(512)
});

/// The version identifier of this build
pub static PKG_VERSION: Lazy<String> = Lazy::new(|| match option_env!("MYNES_BUILD_METADATA") {
	Some(metadata) if !metadata.trim().is_empty() => {
		let version = env!("CARGO_PKG_VERSION");
		format!("{version}+{metadata}")
	}
	_ => env!("CARGO_PKG_VERSION").to_owned(),
});
