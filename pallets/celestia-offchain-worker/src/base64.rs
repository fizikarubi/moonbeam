use alloc::string::String;
use data_encoding::BASE64;
use serde::Serializer;
use sp_std::vec;

pub fn serialize_to_base64<S: Serializer>(
	input: &[u8],
	serializer: S,
) -> core::result::Result<S::Ok, S::Error> {
	let base64 = encode(input);
	serializer.serialize_str(&base64)
}

// Encode byte array to base64 string representation.
fn encode(input: &[u8]) -> String {
	let mut output = vec![0; BASE64.encode_len(input.len())];
	BASE64.encode_mut(input, &mut output);
	// In Base64 encoding, the output is always valid ASCII (which is a subset of UTF-8)
	// so we can safely convert the output
	unsafe { alloc::string::String::from_utf8_unchecked(output) }
}
