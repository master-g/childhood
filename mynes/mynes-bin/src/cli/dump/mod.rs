use clap::{Args, Subcommand};

use crate::err::Error;

mod chr;
mod rom;

#[derive(Debug, Subcommand)]
enum Commands {
	#[command(about = "dump chr")]
	Chr(chr::ChrCommandArguments),

	#[command(about = "dump rom")]
	Rom(rom::RomCommandArguments),
}

#[derive(Args, Debug)]
pub(super) struct DumpCommandArguments {
	#[command(subcommand)]
	command: Commands,
}

pub(super) async fn init(args: DumpCommandArguments) -> Result<(), Error> {
	match args.command {
		Commands::Chr(args) => chr::exec(args).await,
		Commands::Rom(args) => rom::exec(args).await,
	}
}
