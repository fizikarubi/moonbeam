use super::base64::serialize_to_base64;
use super::commitment::{self, Commitment, Namespace};
use super::error::Error;
use frame_support::sp_runtime::offchain::http;
use scale_info::prelude::string::String;
use serde::{Deserialize, Serialize};
use sp_std::{vec, vec::Vec};

/// # Examples
///
/// ```
///
/// let request_body = br#'{
///     "id": 1,
///     "jsonrpc": "2.0",
///     "method": "blob.Submit",
///     "params": [
///         [
///             {
///                 "namespace": "AAAAAAAAAAAAAAAAAAAAAAAAAAECAwQFBgcICRA=",
///                 "data": "VGhpcyBpcyBhbiBleGFtcGxlIG9mIHNvbWUgYmxvYiBkYXRh",
///                 "share_version": 0,
///                 "commitment": "AD5EzbG0/EMvpw0p8NIjMVnoCP4Bv6K+V6gjmwdXUKU="
///             }
///         ],
///         {
///            "Fee": 2000,
///             "GasLimit": 200000
///         }
///     ]
///  }'#;
///
/// let request = build_rpc_request("http://localhost:3000", request_body, "...")
/// ```
///
///
pub fn build_rpc_request<'a, T: AsRef<[u8]>>(
	url: &'a str,
	request_body: T,
	jwt: &'a str, // Celestia node requires JWT for read/write permission
) -> http::Request<'a, Vec<T>> {
	http::Request::post(url, vec![request_body])
		.add_header("Content-Type", "application/json")
		.add_header("Authorization", &(String::from("Bearer ") + jwt))
}

// 0x0000006672616374616c in hex.
const NAMESPACE: &str = "fractal";
const SHARED_VERSION: u8 = 0;

#[derive(Serialize)]
pub struct SubmitParam<'a> {
	namespace: Namespace,
	#[serde(serialize_with = "serialize_to_base64")]
	data: &'a [u8],
	shared_version: u8,
	commitment: Commitment,
}

impl<'a> sp_std::convert::TryFrom<&'a [u8]> for SubmitParam<'a> {
	type Error = commitment::Error;
	fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
		let namespace = Namespace::new_v0(NAMESPACE.as_bytes())?;

		let commitment = Commitment::from_blob(namespace, SHARED_VERSION, data)?;

		Ok(Self {
			namespace,
			data,
			shared_version: SHARED_VERSION,
			commitment,
		})
	}
}

#[derive(Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct GasParam {
	fee: u64,
	gas_limit: u64,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum ParamValue<'a> {
	Submit(Vec<SubmitParam<'a>>),
	Gas(GasParam),
}

#[derive(Serialize)]
pub(crate) struct RpcRequestBody<'a> {
	id: String,
	jsonrpc: String,
	method: String,
	params: Vec<ParamValue<'a>>,
}

impl<'a> RpcRequestBody<'a> {
	fn new(method: &str, params: Vec<ParamValue<'a>>) -> Self {
		RpcRequestBody {
			id: String::from("1"),
			jsonrpc: String::from("2.0"),
			method: method.into(),
			params,
		}
	}

	// Build request body for `blob.Submit`` method
	// https://node-rpc-docs.celestia.org/?version=v0.12.1#blob.Submit
	pub fn blob_submit(submit_param: SubmitParam<'a>) -> core::result::Result<Self, Error> {
		// TODO: dynamically calcualte the gas fee
		let gas = GasParam {
			fee: 2000,
			gas_limit: 200000,
		};

		Ok(RpcRequestBody::new(
			"blob.Submit",
			vec![ParamValue::Submit(vec![submit_param]), ParamValue::Gas(gas)],
		))
	}

	pub fn try_to_string(&self) -> serde_json::Result<String> {
		serde_json::to_string(self)
	}
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct RpcResponse {
	// the block height at which the blob was submitted to
	pub result: u64,
}
