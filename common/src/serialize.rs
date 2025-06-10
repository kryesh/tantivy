use std::borrow::Cow;
use std::io::Write;
use std::{fmt, io};

use byteorder::{ByteOrder, WriteBytesExt};

use crate::{Endianness, VInt};

#[derive(Default)]
struct Counter(u64);

impl io::Write for Counter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0 += buf.len() as u64;
        Ok(buf.len())
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.0 += buf.len() as u64;
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn end_of_buffer_error<T>() -> Result<T, io::Error> {
    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "Unexpected end of buffer",
    ))
}

/// Trait for a simple binary serialization.
pub trait BinarySerializable: fmt::Debug + Sized {
    /// Serialize
    fn serialize<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()>;

    fn num_bytes(&self) -> u64 {
        let mut counter = Counter::default();
        self.serialize(&mut counter).unwrap();
        counter.0
    }
}

pub trait BinaryDeserializable<'de>: Sized {
    /// Returns the deserialized object and the number of bytes read.
    fn deserialize(buf: &'de [u8]) -> io::Result<(Self, usize)>;
}

/// `FixedSize` marks a `BinarySerializable` as
/// always serializing to the same size.
pub trait FixedSize: BinarySerializable {
    const SIZE_IN_BYTES: usize;
}

impl BinarySerializable for () {
    fn serialize<W: Write + ?Sized>(&self, _: &mut W) -> io::Result<()> {
        Ok(())
    }
}

impl BinaryDeserializable<'_> for () {
    fn deserialize(_buf: &[u8]) -> io::Result<(Self, usize)> {
        Ok(((), 0))
    }
}

impl FixedSize for () {
    const SIZE_IN_BYTES: usize = 0;
}

impl<T: BinarySerializable> BinarySerializable for Vec<T> {
    fn serialize<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
        BinarySerializable::serialize(&VInt(self.len() as u64), writer)?;
        for it in self {
            it.serialize(writer)?;
        }
        Ok(())
    }
}

impl<T: for<'de> BinaryDeserializable<'de>> BinaryDeserializable<'_> for Vec<T> {
    fn deserialize(buf: &[u8]) -> io::Result<(Vec<T>, usize)> {
        let (num_items, mut bytes_read) =
            VInt::deserialize(buf).map(|(v, bytes)| (v.val() as usize, bytes))?;
        let mut items: Vec<T> = Vec::with_capacity(num_items);
        for _ in 0..num_items {
            if let Some(slice) = buf.get(bytes_read..) {
                let (item, item_bytes) = T::deserialize(slice)?;
                items.push(item);
                bytes_read += item_bytes;
            } else {
                return end_of_buffer_error();
            }
        }
        Ok((items, bytes_read))
    }
}

impl<Left: BinarySerializable, Right: BinarySerializable> BinarySerializable for (Left, Right) {
    fn serialize<W: Write + ?Sized>(&self, write: &mut W) -> io::Result<()> {
        self.0.serialize(write)?;
        self.1.serialize(write)
    }
}
impl<Left: BinarySerializable + FixedSize, Right: BinarySerializable + FixedSize> FixedSize
    for (Left, Right)
{
    const SIZE_IN_BYTES: usize = Left::SIZE_IN_BYTES + Right::SIZE_IN_BYTES;
}

impl BinarySerializable for u32 {
    fn serialize<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_u32::<Endianness>(*self)
    }
}

impl BinaryDeserializable<'_> for u32 {
    fn deserialize(buf: &[u8]) -> io::Result<(Self, usize)> {
        if buf.len() >= size_of::<Self>() {
            let value = Endianness::read_u32(buf);
            Ok((value, size_of::<Self>()))
        } else {
            end_of_buffer_error()
        }
    }
}

impl FixedSize for u32 {
    const SIZE_IN_BYTES: usize = 4;
}

impl BinarySerializable for u16 {
    fn serialize<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_u16::<Endianness>(*self)
    }
}

impl BinaryDeserializable<'_> for u16 {
    fn deserialize(buf: &[u8]) -> io::Result<(Self, usize)> {
        if buf.len() >= size_of::<Self>() {
            let value = Endianness::read_u16(buf);
            Ok((value, size_of::<Self>()))
        } else {
            end_of_buffer_error()
        }
    }
}

impl FixedSize for u16 {
    const SIZE_IN_BYTES: usize = 2;
}

impl BinarySerializable for u64 {
    fn serialize<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_u64::<Endianness>(*self)
    }
}

impl BinaryDeserializable<'_> for u64 {
    fn deserialize(buf: &[u8]) -> io::Result<(Self, usize)> {
        if buf.len() >= size_of::<Self>() {
            let value = Endianness::read_u64(buf);
            Ok((value, size_of::<Self>()))
        } else {
            end_of_buffer_error()
        }
    }
}

impl FixedSize for u64 {
    const SIZE_IN_BYTES: usize = 8;
}

impl BinarySerializable for u128 {
    fn serialize<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_u128::<Endianness>(*self)
    }
}

impl BinaryDeserializable<'_> for u128 {
    fn deserialize(buf: &[u8]) -> io::Result<(Self, usize)> {
        if buf.len() >= size_of::<Self>() {
            let value = Endianness::read_u128(buf);
            Ok((value, size_of::<Self>()))
        } else {
            end_of_buffer_error()
        }
    }
}

impl FixedSize for u128 {
    const SIZE_IN_BYTES: usize = 16;
}

impl BinarySerializable for f32 {
    fn serialize<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_f32::<Endianness>(*self)
    }
}

impl BinaryDeserializable<'_> for f32 {
    fn deserialize(buf: &[u8]) -> io::Result<(Self, usize)> {
        if buf.len() >= size_of::<Self>() {
            let value = Endianness::read_f32(buf);
            Ok((value, size_of::<Self>()))
        } else {
            end_of_buffer_error()
        }
    }
}

impl FixedSize for f32 {
    const SIZE_IN_BYTES: usize = 4;
}

impl BinarySerializable for i64 {
    fn serialize<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_i64::<Endianness>(*self)
    }
}

impl BinaryDeserializable<'_> for i64 {
    fn deserialize(buf: &[u8]) -> io::Result<(Self, usize)> {
        if buf.len() >= size_of::<Self>() {
            let value = Endianness::read_i64(buf);
            Ok((value, size_of::<Self>()))
        } else {
            end_of_buffer_error()
        }
    }
}

impl FixedSize for i64 {
    const SIZE_IN_BYTES: usize = 8;
}

impl BinarySerializable for f64 {
    fn serialize<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_f64::<Endianness>(*self)
    }
}

impl BinaryDeserializable<'_> for f64 {
    fn deserialize(buf: &[u8]) -> io::Result<(Self, usize)> {
        if buf.len() >= size_of::<Self>() {
            let value = Endianness::read_f64(buf);
            Ok((value, size_of::<Self>()))
        } else {
            end_of_buffer_error()
        }
    }
}

impl FixedSize for f64 {
    const SIZE_IN_BYTES: usize = 8;
}

impl BinarySerializable for u8 {
    fn serialize<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_u8(*self)
    }
}

impl BinaryDeserializable<'_> for u8 {
    fn deserialize(buf: &[u8]) -> io::Result<(Self, usize)> {
        if !buf.is_empty() {
            Ok((buf[0], 1))
        } else {
            end_of_buffer_error()
        }
    }
}

impl FixedSize for u8 {
    const SIZE_IN_BYTES: usize = 1;
}

impl BinarySerializable for bool {
    fn serialize<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_u8(u8::from(*self))
    }
}

impl BinaryDeserializable<'_> for bool {
    fn deserialize(buf: &[u8]) -> io::Result<(Self, usize)> {
        if !buf.is_empty() {
            match buf[0] {
                0 => Ok((false, 1)),
                1 => Ok((true, 1)),
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid bool value on deserialization, data corrupted",
                )),
            }
        } else {
            end_of_buffer_error()
        }
    }
}

impl FixedSize for bool {
    const SIZE_IN_BYTES: usize = 1;
}

impl BinarySerializable for String {
    fn serialize<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
        let data: &[u8] = self.as_bytes();
        BinarySerializable::serialize(&VInt(data.len() as u64), writer)?;
        writer.write_all(data)
    }
}

impl BinaryDeserializable<'_> for String {
    fn deserialize(buf: &[u8]) -> io::Result<(Self, usize)> {
        let (vint, vint_size) = VInt::deserialize(buf)?;
        let string_length = vint.val() as usize;
        let start = vint_size;
        let end = start + string_length;

        if let Some(slice) = buf.get(start..end) {
            let result = String::from_utf8(slice.to_vec()).map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 sequence")
            })?;
            Ok((result, end))
        } else {
            end_of_buffer_error()
        }
    }
}

impl BinarySerializable for &str {
    fn serialize<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
        let data: &[u8] = self.as_bytes();
        BinarySerializable::serialize(&VInt(data.len() as u64), writer)?;
        writer.write_all(data)
    }
}

impl<'de> BinaryDeserializable<'de> for &'de str {
    fn deserialize(buf: &'de [u8]) -> io::Result<(Self, usize)> {
        let (vint, vint_size) = VInt::deserialize(buf)?;
        let string_length = vint.val() as usize;
        let start = vint_size;
        let end = start + string_length;

        if let Some(slice) = buf.get(start..end) {
            let result = str::from_utf8(slice).map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 sequence")
            })?;
            Ok((result, end))
        } else {
            end_of_buffer_error()
        }
    }
}

impl<'a> BinarySerializable for Cow<'a, str> {
    fn serialize<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
        let data: &[u8] = self.as_bytes();
        BinarySerializable::serialize(&VInt(data.len() as u64), writer)?;
        writer.write_all(data)
    }
}

impl<'de> BinaryDeserializable<'de> for Cow<'de, str> {
    fn deserialize(buf: &'de [u8]) -> io::Result<(Self, usize)> {
        let (vint, vint_size) = VInt::deserialize(buf)?;
        let string_length = vint.val() as usize;
        let start = vint_size;
        let end = start + string_length;

        if let Some(slice) = buf.get(start..end) {
            match str::from_utf8(slice) {
                Ok(valid_str) => Ok((Cow::Borrowed(valid_str), end)),
                Err(_) => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid UTF-8 sequence",
                )),
            }
        } else {
            end_of_buffer_error()
        }
    }
}

impl<'a> BinarySerializable for Cow<'a, [u8]> {
    fn serialize<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
        BinarySerializable::serialize(&VInt(self.len() as u64), writer)?;
        for it in self.iter() {
            BinarySerializable::serialize(it, writer)?;
        }
        Ok(())
    }
}

impl<'de> BinaryDeserializable<'de> for Cow<'de, [u8]> {
    fn deserialize(buf: &'de [u8]) -> io::Result<(Self, usize)> {
        let (vint, vint_size) = VInt::deserialize(buf)?;
        let len = vint.val();
        let start = vint_size;
        let end = start + len as usize;

        if let Some(slice) = buf.get(start..end) {
            Ok((Cow::Borrowed(slice), end))
        } else {
            end_of_buffer_error()
        }
    }
}

#[cfg(test)]
pub mod test {

    use super::*;
    pub fn fixed_size_test<O: BinarySerializable + FixedSize + Default>() {
        let mut buffer = Vec::new();
        O::default().serialize(&mut buffer).unwrap();
        assert_eq!(buffer.len(), O::SIZE_IN_BYTES);
    }

    fn serialize_test<T: BinarySerializable + for<'a> BinaryDeserializable<'a> + Eq>(
        v: T,
    ) -> usize {
        let mut buffer: Vec<u8> = Vec::new();
        v.serialize(&mut buffer).unwrap();
        let num_bytes = buffer.len();
        let (deser, bytes_read) = T::deserialize(&buffer).unwrap();
        assert_eq!(deser, v);
        assert_eq!(bytes_read, num_bytes);

        num_bytes
    }

    #[test]
    fn test_serialize_u8() {
        fixed_size_test::<u8>();
    }

    #[test]
    fn test_serialize_u32() {
        fixed_size_test::<u32>();
        assert_eq!(4, serialize_test(3u32));
        assert_eq!(4, serialize_test(5u32));
        assert_eq!(4, serialize_test(u32::MAX));
    }

    #[test]
    fn test_serialize_i64() {
        fixed_size_test::<i64>();
    }

    #[test]
    fn test_serialize_f64() {
        fixed_size_test::<f64>();
    }

    #[test]
    fn test_serialize_u64() {
        fixed_size_test::<u64>();
    }

    #[test]
    fn test_serialize_bool() {
        fixed_size_test::<bool>();
    }

    #[test]
    fn test_serialize_string() {
        assert_eq!(serialize_test(String::from("")), 1);
        assert_eq!(serialize_test(String::from("ぽよぽよ")), 1 + 3 * 4);
        assert_eq!(serialize_test(String::from("富士さん見える。")), 1 + 3 * 8);
    }

    #[test]
    fn test_serialize_vec() {
        assert_eq!(serialize_test(Vec::<u8>::new()), 1);
        assert_eq!(serialize_test(vec![1u32, 3u32]), 1 + 4 * 2);
    }

    #[test]
    fn test_serialize_vint() {
        for i in 0..10_000 {
            serialize_test(VInt(i as u64));
        }
        assert_eq!(serialize_test(VInt(7u64)), 1);
        assert_eq!(serialize_test(VInt(127u64)), 1);
        assert_eq!(serialize_test(VInt(128u64)), 2);
        assert_eq!(serialize_test(VInt(129u64)), 2);
        assert_eq!(serialize_test(VInt(1234u64)), 2);
        assert_eq!(serialize_test(VInt(16_383u64)), 2);
        assert_eq!(serialize_test(VInt(16_384u64)), 3);
        assert_eq!(serialize_test(VInt(u64::MAX)), 10);
    }
}
