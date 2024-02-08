#![allow(dead_code)]
/// The size of the SHA256 hash.
pub const HASH_SIZE: usize = celestia_tendermint::hash::SHA256_HASH_SIZE;

pub mod appconsts {
	pub use global_consts::*;
	pub use v1::*;

	// celestia-app/pkg/appconsts/v1/app_consts
	mod v1 {
		pub const SUBTREE_ROOT_THRESHOLD: u64 = 64;
		pub const SQUARE_SIZE_UPPER_BOUND: usize = 128;
	}

	// celestia-app/pkg/appconsts/global_consts
	mod global_consts {
		const NS_VER_SIZE: usize = 1;
		const NS_ID_SIZE: usize = 28;
		pub const NS_SIZE: usize = NS_VER_SIZE + NS_ID_SIZE;

		/// the size of the namespace.
		pub const NAMESPACE_SIZE: usize = NS_SIZE;

		/// the size of a share in bytes.
		pub const SHARE_SIZE: usize = 512;

		/// the number of bytes reserved for information. The info
		/// byte contains the share version and a sequence start idicator.
		pub const SHARE_INFO_BYTES: usize = 1;

		/// the number of bytes reserved for the sequence length
		/// that is present in the first share of a sequence.
		pub const SEQUENCE_LEN_BYTES: usize = 4;

		/// the first share version format.
		pub const SHARE_VERSION_ZERO: u8 = 0;

		/// the number of bytes reserved for the location of
		/// the first unit (transaction, ISR) in a compact share.
		pub const COMPACT_SHARE_RESERVED_BYTES: usize = 4;

		/// the number of bytes usable for data in the first compact share of a sequence.
		pub const FIRST_COMPACT_SHARE_CONTENT_SIZE: usize = SHARE_SIZE
			- NAMESPACE_SIZE
			- SHARE_INFO_BYTES
			- SEQUENCE_LEN_BYTES
			- COMPACT_SHARE_RESERVED_BYTES;

		/// the number of bytes usable for data in a continuation compact share of a sequence.
		pub const CONTINUATION_COMPACT_SHARE_CONTENT_SIZE: usize =
			SHARE_SIZE - NAMESPACE_SIZE - SHARE_INFO_BYTES - COMPACT_SHARE_RESERVED_BYTES;

		/// the number of bytes usable for data in the first sparse share of a sequence.
		pub const FIRST_SPARSE_SHARE_CONTENT_SIZE: usize =
			SHARE_SIZE - NAMESPACE_SIZE - SHARE_INFO_BYTES - SEQUENCE_LEN_BYTES;

		/// the number of bytes usable for data in a continuation sparse share of a sequence.
		pub const CONTINUATION_SPARSE_SHARE_CONTENT_SIZE: usize =
			SHARE_SIZE - NAMESPACE_SIZE - SHARE_INFO_BYTES;

		/// the smallest original square width.
		pub const MIN_SQUARE_SIZE: usize = 1;

		/// the minimum number of shares allowed in the original data square.
		pub const MIN_SHARE_COUNT: usize = MIN_SQUARE_SIZE * MIN_SQUARE_SIZE;

		/// the maximum value a share version can be.
		pub const MAX_SHARE_VERSION: u8 = 127;
	}
}
