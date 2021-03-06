use std::io::Result as Res;
use std::io::Write;

use crate::{Endianness, EWrite};

/**
	Implement this for your types to be able to `write` them.

	## Examples

	### Serialize a struct:

	Note how the trait bound for `W` is `EWrite<E>`, as we want to use the functionality of this crate to delegate serialization to the struct's fields.

	Note: As you can see below, you may need to write `where` clauses when delegating functionality to other `write` operations. There are two reasons for this:

	- Rust currently can't recognize sealed traits. Even though there are only two endiannesses, and the primitive types are implemented for them, the compiler can't recognize that. If/When the compiler gets smarter about sealed traits this will be resolved. Alternatively, once Rust gets support for specialization, I will be able to add a dummy blanket `impl` to primitives which will work around this issue.

	- The underlying `W` type needs to implement `std::io::Write` to be able to write primitive types. You can work around this by explicitly specifying `Write` as trait bound, but since both `Write` and `EWrite` have a `write` method, Rust will force you to use UFCS syntax to disambiguate between them. This makes using `write` less ergonomic, and I personally think that `where` clauses are the better alternative here, since they avoid this issue.

		Ideally I'd like to make `std::io::Write` a supertrait of `EWrite`, since the serialization will normally depend on `Write` anyway. Unfortunately, supertraits' methods automatically get brought into scope, so this would mean that you would be forced to use UFCS every time, without being able to work around them with `where` clauses. ([Rust issue #17151](https://github.com/rust-lang/rust/issues/17151)).
	```
	struct Example {
		a: u8,
		b: bool,
		c: u32,
	}
	{
		use std::io::{Result, Write};
		use endio::{Endianness, EWrite, Serialize};

		impl<E: Endianness, W: Write + EWrite<E>> Serialize<E, W> for &Example {
			fn serialize(self, writer: &mut W) -> Result<()> {
				writer.ewrite(self.a)?;
				writer.ewrite(self.b)?;
				writer.ewrite(self.c)
			}
		}
	}
	// will then allow you to directly write:
	{
		use endio::LEWrite;

		let mut writer = vec![];
		let e = Example { a: 42, b: true, c: 754187983 };
		writer.ewrite(&e);

		assert_eq!(writer, b"\x2a\x01\xcf\xfe\xf3\x2c");
	}
	# {
	# 	use endio::BEWrite;
	# 	let mut writer = vec![];
	# 	let e = Example { a: 42, b: true, c: 754187983 };
	# 	writer.ewrite(&e);
	# 	assert_eq!(writer, b"\x2a\x01\x2c\xf3\xfe\xcf");
	# }
	```

	### Serialize a primitive / something where you need to use the bare `std::io::Write` functionality:

	Note how the trait bound for `W` is `Write`.
	```
	use std::io::{Result, Write};
	use endio::{Endianness, EWrite, Serialize};

	struct new_u8(u8);

	impl<E: Endianness, W: Write> Serialize<E, W> for &new_u8 {
		fn serialize(self, writer: &mut W) -> Result<()> {
			let mut buf = [0; 1];
			buf[0] = self.0;
			writer.write_all(&buf);
			Ok(())
		}
	}
	```

	### Serialize with endian-specific code:

	Note how instead of using a trait bound on Endianness, we implement Serialize twice, once for `BigEndian` and once for `LittleEndian`.
	```
	use std::io::{Result, Write};
	use std::mem::size_of;
	use endio::{BigEndian, Serialize, LittleEndian};

	struct new_u16(u16);

	impl<W: Write> Serialize<BigEndian, W> for new_u16 {
		fn serialize(self, writer: &mut W) -> Result<()> {
			let mut buf = [0; size_of::<u16>()];
			writer.write_all(&self.0.to_be_bytes())?;
			Ok(())
		}
	}

	impl<W: Write> Serialize<LittleEndian, W> for new_u16 {
		fn serialize(self, writer: &mut W) -> Result<()> {
			writer.write_all(&self.0.to_le_bytes())?;
			Ok(())
		}
	}
	```
*/
pub trait Serialize<E: Endianness, W>: Sized {
	/// Serializes the type by writing to the writer using Big-endian.
	/// Implement ONLY this method if your code for both endianness is the same.
	fn serialize(self, _writer: &mut W) -> Res<()> {
		unreachable!();
	}

	/// Serializes the type by writing to the writer using Big-endian.
	fn serialize_be(self, writer: &mut W) -> Res<()> {
		self.serialize(writer)
	}

	/// Serializes the type by writing to the writer using Little-endian.
	fn serialize_le(self, writer: &mut W) -> Res<()> {
		self.serialize(writer)
	}
}

// todo[specialization]: specialize for &[u8] (std::io::Write::write_all)
/// Writes the entire contents of the byte slice.
impl<E: Endianness, W: EWrite<E>, S: Copy+Serialize<E, W>> Serialize<E, W> for &[S] {
	fn serialize(self, writer: &mut W) -> Res<()> {
		for elem in self {
			writer.ewrite(*elem)?;
		}
		Ok(())
	}
}

/// Writes the entire contents of the Vec.
impl<E: Endianness, W: EWrite<E>, S: Copy+Serialize<E, W>> Serialize<E, W> for &Vec<S> {
	fn serialize(self, writer: &mut W) -> Res<()> {
		writer.ewrite(self.as_slice())
	}
}

/// Writes a bool by writing a byte.
impl<E: Endianness, W: Write> Serialize<E, W> for bool {
	fn serialize(self, writer: &mut W) -> Res<()> {
		writer.write_all(&(self as u8).to_ne_bytes())
	}
}

impl<E: Endianness, W: Write> Serialize<E, W> for u8 {
	fn serialize(self, writer: &mut W) -> Res<()> {
		writer.write_all(&self.to_ne_bytes())
	}
}

impl<E: Endianness, W: Write> Serialize<E, W> for i8 {
	fn serialize(self, writer: &mut W) -> Res<()> {
		writer.write_all(&self.to_ne_bytes())
	}
}

macro_rules! impl_int {
	($t:ident) => {
		impl<E: Endianness, W: Write> Serialize<E, W> for $t {
			fn serialize_be(self, writer: &mut W) -> Res<()> {
				writer.write_all(&self.to_be_bytes())
			}

			fn serialize_le(self, writer: &mut W) -> Res<()> {
				writer.write_all(&self.to_le_bytes())
			}
		}

		#[cfg(test)]
		mod $t {
			use std::mem::size_of;

			#[test]
			fn test() {
				let integer: u128 = 0xbaadf00dbaadf00dbaadf00dbaadf00d;
				let bytes = b"\x0d\xf0\xad\xba\x0d\xf0\xad\xba\x0d\xf0\xad\xba\x0d\xf0\xad\xba";

				{
					use crate::BEWrite;
					let mut writer = vec![];
					writer.ewrite((integer as $t).to_be()).unwrap();
					assert_eq!(&writer[..], &bytes[..size_of::<$t>()]);
				}
				{
					use crate::LEWrite;
					let mut writer = vec![];
					writer.ewrite((integer as $t).to_le()).unwrap();
					assert_eq!(&writer[..], &bytes[..size_of::<$t>()]);
				}
			}
		}
	}
}

impl_int!(u16);
impl_int!(u32);
impl_int!(u64);
impl_int!(u128);
impl_int!(i16);
impl_int!(i32);
impl_int!(i64);
impl_int!(i128);

impl<E: Endianness, W: EWrite<E>> Serialize<E, W> for f32 where u32: Serialize<E, W> {
	fn serialize(self, writer: &mut W) -> Res<()> {
		writer.ewrite(self.to_bits())
	}
}

impl<E: Endianness, W: EWrite<E>> Serialize<E, W> for f64 where u64: Serialize<E, W> {
	fn serialize(self, writer: &mut W) -> Res<()> {
		writer.ewrite(self.to_bits())
	}
}

#[cfg(test)]
mod tests {
	use std::io::Result as Res;

	#[test]
	fn write_slice() {
		let data = b"\xba\xad\xba\xad";
		use crate::LEWrite;
		let mut writer = vec![];
		writer.ewrite(&[0xadbau16, 0xadbau16][..]).unwrap();
		assert_eq!(writer, data);
	}

	#[test]
	fn write_vec() {
		let data = b"\xba\xad\xba\xad";
		use crate::LEWrite;
		let mut writer = vec![];
		writer.ewrite(&vec![0xadbau16, 0xadbau16]).unwrap();
		assert_eq!(writer, data);
	}

	#[test]
	fn write_bool_false() {
		let data = b"\x00";
		let val = false;
		{
			use crate::BEWrite;
			let mut writer = vec![];
			writer.ewrite(val).unwrap();
			assert_eq!(writer, data);
		}
		{
			use crate::LEWrite;
			let mut writer = vec![];
			writer.ewrite(val).unwrap();
			assert_eq!(writer, data);
		}
	}

	#[test]
	fn write_bool_true() {
		let data = b"\x01";
		let val = true;
		{
			use crate::BEWrite;
			let mut writer = vec![];
			writer.ewrite(val).unwrap();
			assert_eq!(writer, data);
		}
		{
			use crate::LEWrite;
			let mut writer = vec![];
			writer.ewrite(val).unwrap();
			assert_eq!(writer, data);
		}
	}

	#[test]
	fn write_i8() {
		let data = b"\x80";
		let val = i8::min_value();
		{
			use crate::BEWrite;
			let mut writer = vec![];
			writer.ewrite(val).unwrap();
			assert_eq!(writer, data);
		}
		{
			use crate::LEWrite;
			let mut writer = vec![];
			writer.ewrite(val).unwrap();
			assert_eq!(writer, data);
		}
	}

	#[test]
	fn write_u8() {
		let data = b"\xff";
		let val = u8::max_value();
		{
			use crate::BEWrite;
			let mut writer = vec![];
			writer.ewrite(val).unwrap();
			assert_eq!(writer, data);
		}
		{
			use crate::LEWrite;
			let mut writer = vec![];
			writer.ewrite(val).unwrap();
			assert_eq!(writer, data);
		}
	}

	#[test]
	fn write_f32() {
		let data = b"\x44\x20\xa7\x44";
		{
			use crate::BEWrite;
			let mut writer = vec![];
			writer.ewrite(642.613525390625f32).unwrap();
			assert_eq!(writer, data);
		}
		{
			use crate::LEWrite;
			let mut writer = vec![];
			writer.ewrite(1337.0083007812f32).unwrap();
			assert_eq!(writer, data);
		}
	}

	#[test]
	fn write_f64() {
		let data = b"\x40\x94\x7a\x14\xae\xe5\x94\x40";
		{
			use crate::BEWrite;
			let mut writer = vec![];
			writer.ewrite(1310.5201984283194f64).unwrap();
			assert_eq!(writer, data);
		}
		{
			use crate::LEWrite;
			let mut writer = vec![];
			writer.ewrite(1337.4199999955163f64).unwrap();
			assert_eq!(writer, data);
		}
	}

	#[test]
	fn write_struct_forced() {
		struct Test {
			a: u16,
		}
		{
			use crate::{Endianness, EWrite, Serialize};

			impl<E: Endianness, W: EWrite<E>> Serialize<E, W> for Test where u16: Serialize<E, W> {
				fn serialize(self, writer: &mut W) -> Res<()> {
					writer.ewrite(self.a)?;
					Ok(())
				}
			}
		}

		use crate::LEWrite;
		let data = b"\xba\xad";
		let mut writer = vec![];
		writer.write_be(Test { a: 0xbaad }).unwrap();
		assert_eq!(&writer[..], data);
	}
}
