mod namespaced_hash;

use nmt_rs::simple_merkle::db::MemDb;
use serde::Serialize;
pub type NamespacedHash = nmt_rs::NamespacedHash<NS_SIZE>;
use crate::base64::serialize_to_base64;

pub use self::namespaced_hash::{NamespacedHashExt, RawNamespacedHash, NAMESPACED_HASH_SIZE};
pub use super::error::{Error, Result};

pub const NS_VER_SIZE: usize = 1;
pub const NS_ID_SIZE: usize = 28;
pub const NS_SIZE: usize = NS_VER_SIZE + NS_ID_SIZE;
pub const NS_ID_V0_SIZE: usize = 10;
pub use super::consts::HASH_SIZE;

pub type NamespacedSha2Hasher = nmt_rs::NamespacedSha2Hasher<NS_SIZE>;
pub type Nmt = nmt_rs::NamespaceMerkleTree<MemDb<NamespacedHash>, NamespacedSha2Hasher, NS_SIZE>;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Namespace(nmt_rs::NamespaceId<NS_SIZE>);

impl Serialize for Namespace {
	fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serialize_to_base64(self.as_bytes(), serializer)
	}
}

impl Namespace {
	pub const TRANSACTION: Namespace = Namespace::const_v0([0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
	pub const PAY_FOR_BLOB: Namespace = Namespace::const_v0([0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
	pub const PRIMARY_RESERVED_PADDING: Namespace = Namespace::MAX_PRIMARY_RESERVED;
	pub const MAX_PRIMARY_RESERVED: Namespace =
		Namespace::const_v0([0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff]);
	pub const MIN_SECONDARY_RESERVED: Namespace = Namespace::const_v255(0);
	pub const TAIL_PADDING: Namespace = Namespace::const_v255(0xfe);
	pub const PARITY_SHARE: Namespace = Namespace::const_v255(0xff);

	pub fn from_raw(bytes: &[u8]) -> Result<Self> {
		if bytes.len() != NS_SIZE {
			return Err(Error::InvalidNamespaceSize);
		}

		Namespace::new(bytes[0], &bytes[1..])
	}

	pub fn new(version: u8, id: &[u8]) -> Result<Self> {
		match version {
			0 => Self::new_v0(id),
			255 => Self::new_v255(id),
			n => Err(Error::UnsupportedNamespaceVersion(n)),
		}
	}

	pub fn new_v0(id: &[u8]) -> Result<Self> {
		let id_pos = match id.len() {
			// Allow 28 bytes len
			NS_ID_SIZE => NS_ID_SIZE - NS_ID_V0_SIZE,
			// Allow 10 bytes len or less
			n if n <= NS_ID_V0_SIZE => 0,
			// Anything else is an error
			_ => return Err(Error::InvalidNamespaceSize),
		};

		let prefix = &id[..id_pos];
		let id = &id[id_pos..];

		// Validate that prefix is all zeros
		if prefix.iter().any(|&x| x != 0) {
			return Err(Error::InvalidNamespaceV0);
		}

		let mut bytes = [0u8; NS_SIZE];
		bytes[NS_SIZE - id.len()..].copy_from_slice(id);

		Ok(Namespace(nmt_rs::NamespaceId(bytes)))
	}

	// pub(crate) const fn new_unchecked(bytes: [u8; NS_SIZE]) -> Self {
	// 	Namespace(nmt_rs::NamespaceId(bytes))
	// }

	pub const fn const_v0(id: [u8; NS_ID_V0_SIZE]) -> Self {
		let mut bytes = [0u8; NS_SIZE];
		let start = NS_SIZE - NS_ID_V0_SIZE;

		bytes[start] = id[0];
		bytes[start + 1] = id[1];
		bytes[start + 2] = id[2];
		bytes[start + 3] = id[3];
		bytes[start + 4] = id[4];
		bytes[start + 5] = id[5];
		bytes[start + 6] = id[6];
		bytes[start + 7] = id[7];
		bytes[start + 8] = id[8];
		bytes[start + 9] = id[9];

		Namespace(nmt_rs::NamespaceId(bytes))
	}

	pub const fn const_v255(id: u8) -> Self {
		let mut bytes = [255u8; NS_SIZE];
		bytes[NS_ID_SIZE] = id;
		Namespace(nmt_rs::NamespaceId(bytes))
	}

	pub fn new_v255(id: &[u8]) -> Result<Self> {
		if id.len() != NS_ID_SIZE {
			return Err(Error::InvalidNamespaceSize);
		}

		// safe after the length check
		let (id, prefix) = id.split_last().unwrap();

		if prefix.iter().all(|&x| x == 0xff) {
			Ok(Namespace::const_v255(*id))
		} else {
			Err(Error::InvalidNamespaceV255)
		}
	}

	pub fn as_bytes(&self) -> &[u8] {
		&self.0 .0
	}

	pub fn version(&self) -> u8 {
		self.as_bytes()[0]
	}

	pub fn id(&self) -> &[u8] {
		&self.as_bytes()[1..]
	}

	pub fn id_v0(&self) -> Option<&[u8]> {
		if self.version() == 0 {
			let start = NS_SIZE - NS_ID_V0_SIZE;
			Some(&self.as_bytes()[start..])
		} else {
			None
		}
	}
}

impl From<Namespace> for nmt_rs::NamespaceId<NS_SIZE> {
	fn from(value: Namespace) -> Self {
		value.0
	}
}

impl From<nmt_rs::NamespaceId<NS_SIZE>> for Namespace {
	fn from(value: nmt_rs::NamespaceId<NS_SIZE>) -> Self {
		Namespace(value)
	}
}

impl core::ops::Deref for Namespace {
	type Target = nmt_rs::NamespaceId<NS_SIZE>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
