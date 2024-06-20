use clap::{Args, Subcommand};

use crate::err::Error;

mod chr;

#[derive(Debug, Subcommand)]
enum Commands {
	#[command(about = "dump chr")]
	Chr(chr::ChrCommandArguments),
}

#[derive(Args, Debug)]
pub(super) struct DumpCommandArguments {
	#[command(subcommand)]
	command: Commands,
}

pub(super) async fn init(args: DumpCommandArguments) -> Result<(), Error> {
	match args.command {
		Commands::Chr(args) => chr::exec(args).await,
	}
}
