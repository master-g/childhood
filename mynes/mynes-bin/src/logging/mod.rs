mod env_filter;

use std::fmt::Debug;
use std::path::PathBuf;

use tracing::Level;
use tracing_log::LogTracer;
use tracing_subscriber::{fmt, Layer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};

pub(crate) use self::env_filter::CustomEnvFilter;
pub(crate) use self::env_filter::CustomEnvFilterParser;

const LOG_FILE_NAME_PREFIX: &str = "mynes.log";

#[derive(Debug, Clone)]
pub struct Builder {
	filter: CustomEnvFilter,
	log_to_path: Option<PathBuf>,
}

pub fn builder() -> Builder {
	Builder::default()
}

impl Default for Builder {
	fn default() -> Self {
		Self {
			filter: CustomEnvFilter(EnvFilter::default()),
			log_to_path: None,
		}
	}
}

impl Builder {
	/// Set the log level on the builder
	#[allow(dead_code)]
	pub fn with_log_level(mut self, log_level: &str) -> Self {
		if let Ok(filter) = filter_from_value(log_level) {
			self.filter = CustomEnvFilter(filter);
		}
		self
	}

	/// Set the filter on the builder
	pub fn with_filter(mut self, filter: CustomEnvFilter) -> Self {
		self.filter = filter;
		self
	}

	pub fn with_file_appender(mut self, path: PathBuf) -> Self {
		self.log_to_path = Some(path);
		self
	}

	/// Build a tracing dispatcher with the fmt subscriber (logs) and the chosen tracer subscriber
	pub fn build(self) -> Option<tracing_appender::non_blocking::WorkerGuard> {
		let filter = self.filter;

		LogTracer::builder()
			// .with_max_level(log::LevelFilter::Error)
			.init()
			.expect("LogTracer failed to init");

		let fmt_layer = fmt::layer()
			.with_level(true)
			.with_writer(std::io::stdout)
			.with_filter(filter.clone().0);

		if let Some(path) = self.log_to_path {
			let file_appender = tracing_appender::rolling::daily(path, LOG_FILE_NAME_PREFIX);
			let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
			let file_layer =
				fmt::layer().with_ansi(false).with_writer(non_blocking).with_filter(filter.0);
			let collector = tracing_subscriber::registry().with(fmt_layer).with(file_layer);
			tracing::subscriber::set_global_default(collector).expect("Tracing collect error");

			Some(guard)
		} else {
			let collector = tracing_subscriber::registry().with(fmt_layer);
			tracing::subscriber::set_global_default(collector).expect("Tracing collect error");

			None
		}
	}
}

/// Create an EnvFilter from the given value. If the value is not a valid log level, it will be treated as EnvFilter directives.
pub fn filter_from_value(v: &str) -> Result<EnvFilter, tracing_subscriber::filter::ParseError> {
	match v {
		// Don't show any logs at all
		"none" => Ok(EnvFilter::default()),
		// Check if we should show all log levels
		"full" => Ok(EnvFilter::default().add_directive(Level::TRACE.into())),
		// Otherwise, let's only show errors
		"error" => Ok(EnvFilter::default().add_directive(Level::ERROR.into())),
		// Specify the log level for each code area
		"warn" | "info" | "debug" | "trace" => {
			EnvFilter::builder().parse(format!("error,mynes={v}"))
		}
		// Let's try to parse the custom log level
		_ => EnvFilter::builder().parse(v),
	}
}
