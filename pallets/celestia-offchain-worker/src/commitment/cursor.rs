use bytes::Buf;
use nostd_cursor::cursor::Cursor as InnerCursor;

// A `no_std` compatible cursor implementation wrapping the `Cursor`.
// This struct provides a cursor-like interface for byte manipulation in `no_std` environments,
// leveraging the functionalities from the `bytes` crate.
pub struct Cursor<T>(InnerCursor<T>);

impl<T> Cursor<T> {
	pub fn new(inner: T) -> Self {
		Self(InnerCursor::new(inner))
	}

	pub fn position(&self) -> usize {
		self.0.position()
	}

	pub fn set_position(&mut self, position: usize) {
		self.0.set_position(position)
	}

	pub fn get_ref(&self) -> &T {
		&self.0.get_ref()
	}
}

impl<T: AsRef<[u8]>> Buf for Cursor<T> {
	fn remaining(&self) -> usize {
		let len = self.get_ref().as_ref().len();
		let pos = self.position();

		if pos >= len {
			return 0;
		}

		len - pos as usize
	}

	fn chunk(&self) -> &[u8] {
		let len = self.get_ref().as_ref().len();
		let pos = self.position();

		if pos >= len {
			return &[];
		}

		&self.get_ref().as_ref()[pos as usize..]
	}

	fn advance(&mut self, cnt: usize) {
		let pos = (self.position() as usize)
			.checked_add(cnt)
			.expect("overflow");

		assert!(pos <= self.get_ref().as_ref().len());
		self.set_position(pos);
	}
}
