use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
	#[error("invalid iNES file header")]
	InvalidHeader,
	#[error("unsupported file version")]
	UnsupportedVersion,
}
