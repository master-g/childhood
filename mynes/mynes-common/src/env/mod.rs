use crate::cst::PKG_VERSION;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

fn os() -> &'static str {
	get_cfg!(target_os: "windows", "macos", "ios", "linux", "android", "freebsd", "openbsd", "netbsd");
	target_os()
}
fn arch() -> &'static str {
	get_cfg!(target_arch: "x86", "x86_64", "mips", "powerpc", "powerpc64", "arm", "aarch64");
	target_arch()
}

pub fn init() {
	// Log version
	info!("Running {}", release());
}

#[must_use]
pub fn release() -> String {
	format!("{} for {} on {}", *PKG_VERSION, os(), arch())
}
