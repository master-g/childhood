use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
	#[error("hex error: {0}")]
	Hex(#[from] hex::FromHexError),
	#[error("image error: {0}")]
	Image(#[from] image::ImageError),
	#[error("cannot open the specified file: {0}")]
	Io(#[from] std::io::Error),
}
