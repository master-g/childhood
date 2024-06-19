mod foo;
mod version;

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use mynes_common::{cst::LOGO, env};

use crate::logging::{CustomEnvFilter, CustomEnvFilterParser};

const INFO: &str = r"
MyNES toolkit command-line interface
";

#[derive(Parser, Debug)]
#[command(name = "MyNES command-line interface", bin_name = "mynes-cli")]
#[command(author, version, about = INFO, before_help = LOGO)]
#[command(disable_version_flag = true, arg_required_else_help = true)]
struct Cli {
	#[arg(help = "The logging level")]
	#[arg(env = "MYNES_LOG", short = 'l', long = "log")]
	#[arg(default_value = "info")]
	#[arg(value_parser = CustomEnvFilterParser::new())]
	#[arg(global = true)]
	log: CustomEnvFilter,

	#[command(subcommand)]
	command: Option<Commands>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Subcommand)]
enum Commands {
	#[command(about = "Test command")]
	Foo,
	#[command(about = "Print version information")]
	Version,
}

pub(crate) fn prepare(log: CustomEnvFilter) -> Option<tracing_appender::non_blocking::WorkerGuard> {
	crate::logging::builder()
		.with_filter(log)
		.with_file_appender(std::path::PathBuf::from(".logs"))
		.build()
}

pub async fn init() -> ExitCode {
	env::init();

	let args = Cli::parse();

	// version command is special
	if let Some(Commands::Version) = args.command {
		version::init();
		return ExitCode::SUCCESS;
	}

	let _guard = prepare(args.log);

	let output = match args.command {
		Some(Commands::Foo) => foo::init().await,
		_ => Ok(()),
	};

	if let Err(e) = output {
		error!("{}", e);
		ExitCode::FAILURE
	} else {
		ExitCode::SUCCESS
	}
}
