use clap::Args;
use tokio::io::AsyncReadExt;

use crate::err::Error;

#[derive(Args, Debug)]
pub(super) struct RomCommandArguments {
	#[arg(short, long, help = "path to the rom file")]
	src: String,

	#[arg(short, long, help = "output")]
	out: String,
}

pub(super) async fn exec(args: RomCommandArguments) -> Result<(), Error> {
	let mut f = tokio::fs::File::open(&args.src).await?;
	let mut buf = vec![0u8; 16];
	let _ = f.read(buf.as_mut_slice()).await?;
	let header = mynes_rom::header::INesHeaderInfo::new(&buf)?;
	info!("header: {:?}", header);
	Ok(())
}
