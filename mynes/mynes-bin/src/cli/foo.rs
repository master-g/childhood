use crate::err::Error;

#[allow(clippy::unused_async)]
pub(super) async fn init() -> Result<(), Error> {
	trace!("foo!");
	debug!("foo!");
	info!("foo!");
	warn!("foo!");
	error!("foo!");

	println!("foo!");
	Ok(())
}
