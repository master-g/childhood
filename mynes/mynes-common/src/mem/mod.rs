#![allow(missing_docs)]

#[cfg(target_os = "android")]
#[global_allocator]
pub static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_os = "freebsd")]
#[global_allocator]
pub static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_os = "ios")]
#[global_allocator]
pub static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(target_os = "linux")]
#[global_allocator]
pub static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(target_os = "macos")]
#[global_allocator]
pub static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(target_os = "netbsd")]
#[global_allocator]
pub static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_os = "openbsd")]
#[global_allocator]
pub static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

// windows should support mimalloc, but it seems to have some issues while cross compiling for window-gnu target
#[cfg(target_os = "windows")]
use std::alloc::System;

#[cfg(target_os = "windows")]
#[global_allocator]
pub static ALLOC: System = System;
