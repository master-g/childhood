use crate::err::Error;

pub(super) async fn init() -> Result<(), Error> {
	trace!("foo!");
	debug!("foo!");
	info!("foo!");
	warn!("foo!");
	error!("foo!");

	println!("foo!");
	Ok(())
}
