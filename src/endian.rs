use crate::{Deserialize, Serialize};
use std::io;

/**
	Only necessary for custom (de-)serializations.

	You can use this as a blanket impl trait bound to write code that is not endian-specific.

	You can't implement this trait, it only exists as a trait bound.
*/
pub trait Endianness: Sized + private::Sealed {
	fn serialize<W, S: Serialize<Self, W>>(value: S, writer: &mut W) -> io::Result<()>;
	fn deserialize<R, D: Deserialize<Self, R>>(reader: &mut R) -> io::Result<D>;
}

/**
	Only necessary for custom (de-)serializations.

	You can use this as a type parameter in your implementation to write code specific to big endian.
*/
pub struct BigEndian;
/**
	Only necessary for custom (de-)serializations.

	You can use this as a type parameter in your implementation to write code specific to little endian.
*/
pub struct LittleEndian;

impl Endianness for BigEndian {
	fn serialize<W, S: Serialize<Self, W>>(value: S, writer: &mut W) -> io::Result<()> {
		value.serialize_be(writer)
	}

	fn deserialize<R, D: Deserialize<Self, R>>(reader: &mut R) -> io::Result<D> {
		D::deserialize_be(reader)
	}
}

impl Endianness for LittleEndian {
	fn serialize<W, S: Serialize<Self, W>>(value: S, writer: &mut W) -> io::Result<()> {
		value.serialize_le(writer)
	}

	fn deserialize<R, D: Deserialize<Self, R>>(reader: &mut R) -> io::Result<D> {
		D::deserialize_le(reader)
	}
}

// ensures no one else implements the trait
mod private {
	pub trait Sealed {}

	impl Sealed for super::BigEndian {}
	impl Sealed for super::LittleEndian {}
}
