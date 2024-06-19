use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
	#[error("bad argument: {0}")]
	BadArgument(String),
	#[error("cannot open the specified file: {0}")]
	Io(#[from] std::io::Error),
	#[error("there was an error with JSON serialization/deserializatoin: {0}")]
	Json(String),
	#[error("the value was not found: {0}")]
	ValueNotFound(String),
	#[error("authentication error: {0}")]
	Auth(String),
	#[error("validate error: {0}")]
	Validate(String),
}
