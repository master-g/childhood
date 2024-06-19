use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
	#[error("cannot open the specified file: {0}")]
	Io(#[from] std::io::Error),
}
