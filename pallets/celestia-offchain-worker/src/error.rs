use crate::commitment;
use frame_support::sp_runtime::offchain::http;
#[derive(Debug)]
pub(crate) enum Error {
	Http(http::Error),
	Serde(serde_json::Error),
	Commitment(commitment::Error),
}

impl From<http::Error> for Error {
	fn from(value: http::Error) -> Self {
		Error::Http(value)
	}
}

impl From<commitment::Error> for Error {
	fn from(value: commitment::Error) -> Self {
		Error::Commitment(value)
	}
}

impl From<serde_json::Error> for Error {
	fn from(value: serde_json::Error) -> Self {
		Error::Serde(value)
	}
}
