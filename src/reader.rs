use crate::{
    endian::{BigEndian, Endian, LittleEndian},
    result::{ReadError, ReadResult},
};
use std::{
    fmt::Debug,
    io::{BufRead, BufReader},
    marker::PhantomData,
};

pub struct BinaryReader<E, R> {
    reader: R,
    total_bytes_read: usize,
    limit_bytes: Option<usize>,
    _endian: PhantomData<fn() -> E>,
}

pub type Result<T> = ReadResult<T>;

impl<R> BinaryReader<LittleEndian, R> {
    /// Creates a new little-endian `BinaryReader` wrapping the given reader.
    pub fn new_le(reader: R) -> Self {
        BinaryReader {
            reader,
            total_bytes_read: 0,
            limit_bytes: None,
            _endian: PhantomData,
        }
    }
}
impl<'a> BinaryReader<LittleEndian, BufReader<&'a [u8]>> {
    /// Creates a new little-endian `BinaryReader` from an in-memory byte slice.
    pub fn from_le_bytes(data: &'a [u8]) -> Self {
        BinaryReader {
            reader: BufReader::new(data),
            total_bytes_read: 0,
            limit_bytes: Some(data.len()),
            _endian: PhantomData,
        }
    }
}
impl<R> BinaryReader<BigEndian, R> {
    /// Creates a new big-endian `BinaryReader` wrapping the given reader.
    pub fn new_be(reader: R) -> Self {
        BinaryReader {
            reader,
            total_bytes_read: 0,
            limit_bytes: None,
            _endian: PhantomData,
        }
    }
}
impl<'a> BinaryReader<BigEndian, BufReader<&'a [u8]>> {
    /// Creates a new big-endian `BinaryReader` from an in-memory byte slice.
    pub fn from_be_bytes(data: &'a [u8]) -> Self {
        BinaryReader {
            reader: BufReader::new(data),
            total_bytes_read: 0,
            limit_bytes: Some(data.len()),
            _endian: PhantomData,
        }
    }
}

impl<E: Endian, R: BufRead> BinaryReader<E, R> {
    /// Checks if `len` bytes can be read without exceeding the limit.
    #[inline]
    fn check_size(&self, len: usize) -> Result<()> {
        if let Some(limit_bytes) = self.limit_bytes {
            if self.total_bytes_read + len > limit_bytes {
                return Err(ReadError::not_enough_bytes(len, limit_bytes - self.total_bytes_read));
            }
        }
        Ok(())
    }

    /// Reads `len` bytes, applies `parse`, and returns the parsed value.
    #[inline]
    pub fn read_from_slice<T>(&mut self, len: usize, parse: impl Fn(&[u8]) -> Result<T>) -> Result<T> {
        self.check_size(len)?;
        const POPULAR_BUF_SIZE: usize = 512;
        if len <= POPULAR_BUF_SIZE {
            let mut buf = [0u8; POPULAR_BUF_SIZE];
            self.reader.read_exact(&mut buf[..len]).map_err(ReadError::io_error)?;
            self.total_bytes_read += len;
            parse(&buf[..len])
        } else {
            let mut buf = vec![0u8; len];
            self.reader.read_exact(&mut buf).map_err(ReadError::io_error)?;
            self.total_bytes_read += len;
            parse(&buf)
        }
    }

    /// Reads exactly `N` bytes into an array and applies `parse`.
    pub fn read_from_array<T, const N: usize>(&mut self, parse: impl Fn(&[u8; N]) -> T) -> Result<T> {
        self.check_size(N)?;
        let mut buf = [0u8; N];
        self.reader.read_exact(&mut buf).map_err(ReadError::io_error)?;
        self.total_bytes_read += N;
        Ok(parse(&buf))
    }

    /// Reads items of size `N` while `try_parse` returns `Some`.
    #[inline]
    pub fn read_while<T, const N: usize>(&mut self, try_parse: impl Fn(&[u8; N]) -> Option<T>) -> Result<Vec<T>> {
        let mut values = Vec::new();
        let mut buf = [0u8; N];
        loop {
            self.check_size(N)?;
            self.reader.read_exact(&mut buf).map_err(ReadError::io_error)?;
            self.total_bytes_read += N;
            match try_parse(&buf) {
                Some(v) => values.push(v),
                None => break,
            }
        }
        Ok(values)
    }

    /// Reads an unsigned 8‑bit integer.
    #[inline]
    pub fn u8(&mut self) -> Result<u8> {
        self.read_from_array(|b: &[u8; 1]| b[0])
    }

    /// Reads a signed 8‑bit integer.
    #[inline]
    pub fn i8(&mut self) -> Result<i8> {
        self.read_from_array(|b: &[u8; 1]| b[0] as i8)
    }

    /// Reads an unsigned 16‑bit integer according to the configured endian.
    #[inline]
    pub fn u16(&mut self) -> Result<u16> {
        self.read_from_array(E::u16_from_bytes)
    }

    /// Reads a signed 16‑bit integer according to the configured endian.
    #[inline]
    pub fn i16(&mut self) -> Result<i16> {
        self.read_from_array(E::i16_from_bytes)
    }

    /// Reads an unsigned 32‑bit integer according to the configured endian.
    #[inline]
    pub fn u32(&mut self) -> Result<u32> {
        self.read_from_array(E::u32_from_bytes)
    }

    /// Reads a signed 32‑bit integer according to the configured endian.
    #[inline]
    pub fn i32(&mut self) -> Result<i32> {
        self.read_from_array(E::i32_from_bytes)
    }

    /// Reads a 32‑bit floating point value according to the configured endian.
    #[inline]
    pub fn f32(&mut self) -> Result<f32> {
        self.read_from_array(E::f32_from_bytes)
    }

    /// Reads an unsigned 64‑bit integer according to the configured endian.
    #[inline]
    pub fn u64(&mut self) -> Result<u64> {
        self.read_from_array(E::u64_from_bytes)
    }

    /// Reads a signed 64‑bit integer according to the configured endian.
    #[inline]
    pub fn i64(&mut self) -> Result<i64> {
        self.read_from_array(E::i64_from_bytes)
    }

    /// Reads a 64‑bit floating point value according to the configured endian.
    #[inline]
    pub fn f64(&mut self) -> Result<f64> {
        self.read_from_array(E::f64_from_bytes)
    }

    /// Reads any value implementing the `Read` trait.
    #[inline]
    pub fn read<T: Read>(&mut self) -> Result<T> {
        T::read(self)
    }

    /// Reads any value implementing the `ReadWith` trait with an argument.
    #[inline]
    pub fn read_with<T: ReadWith<U>, U>(&mut self, arg: U) -> Result<T> {
        T::read_with(self, arg)
    }

    /// Verifies the next value equals `value`.
    #[inline]
    pub fn value<T: PartialEq + Read + Debug>(&mut self, value: &T) -> Result<()> {
        if self.read::<T>()? != *value {
            return Err(ReadError::invalid_data_format(format!("Expected value to be {:?}, but got something else", value)));
        }
        Ok(())
    }

    /// Verifies the next sequence equals `values`.
    #[inline]
    pub fn values<T: PartialEq + Read + Debug>(&mut self, values: &[T]) -> Result<()> {
        for value in values {
            self.value(value)?;
        }
        Ok(())
    }

    /// Ensures the next `N` bytes all equal `expected_value`.
    #[inline]
    pub fn reserved<const N: usize>(&mut self, expected_value: u8) -> Result<()> {
        self.read_from_array(|data: &[u8; N]| {
            for &byte in data.iter() {
                if byte != expected_value {
                    return Err(ReadError::invalid_data_format(
                        format!("Expected reserved byte to be 0x{:02X}, but got 0x{:02X}", expected_value, byte,),
                    ));
                }
            }
            Ok(())
        })?
    }

    /// Reads a sub-structure of length `len`.
    #[inline]
    pub fn read_partial<T: Read>(&mut self, len: usize) -> Result<T> {
        let original_limit = self.limit_bytes;
        self.limit_bytes = Some(self.total_bytes_read + len);
        let result = T::read(self)?;
        self.limit_bytes = original_limit;
        Ok(result)
    }

    /// Reads a sub-structure with argument `arg` and length `len`.
    #[inline]
    pub fn read_partial_with<U, T: ReadWith<U>>(&mut self, len: usize, arg: U) -> Result<T> {
        let original_limit = self.limit_bytes;
        self.limit_bytes = Some(self.total_bytes_read + len);
        let result = T::read_with(self, arg)?;
        self.limit_bytes = original_limit;
        Ok(result)
    }

    /// Skips `bytes` bytes.
    #[inline]
    pub fn skip(&mut self, bytes: usize) -> Result<()> {
        self.read_from_slice(bytes, |_| Ok(()))
    }

    /// Skips up to alignment boundary by padding.
    #[inline]
    pub fn skip_aligned(&mut self, align: usize) -> Result<()> {
        let offset = self.total_bytes_read % align;
        if offset != 0 {
            self.skip(align - offset)?;
        }
        Ok(())
    }
}

pub trait Read {
    fn read<E: Endian, R: BufRead>(reader: &mut BinaryReader<E, R>) -> Result<Self>
    where
        Self: Sized;
}

pub trait ReadWith<A> {
    fn read_with<E: Endian, R: BufRead>(reader: &mut BinaryReader<E, R>, arg: A) -> Result<Self>
    where
        Self: Sized;
}

impl<R: BufRead> BinaryReader<LittleEndian, R> {
    pub fn read_as_be<T: Read>(&mut self) -> Result<T> {
        let mut be_reader = BinaryReader {
            reader: &mut self.reader,
            total_bytes_read: self.total_bytes_read,
            limit_bytes: self.limit_bytes,
            _endian: PhantomData::<fn() -> BigEndian>,
        };
        let v = T::read(&mut be_reader)?;
        self.total_bytes_read = be_reader.total_bytes_read;
        Ok(v)
    }
    pub fn read_as_be_with<T: ReadWith<U>, U>(&mut self, arg: U) -> Result<T> {
        let mut be_reader = BinaryReader {
            reader: &mut self.reader,
            total_bytes_read: self.total_bytes_read,
            limit_bytes: self.limit_bytes,
            _endian: PhantomData::<fn() -> BigEndian>,
        };
        let v = T::read_with(&mut be_reader, arg)?;
        self.total_bytes_read = be_reader.total_bytes_read;
        Ok(v)
    }
}

impl<R: BufRead> BinaryReader<BigEndian, R> {
    pub fn read_as_le<T: Read>(&mut self) -> Result<T> {
        let mut le_reader = BinaryReader {
            reader: &mut self.reader,
            total_bytes_read: self.total_bytes_read,
            limit_bytes: self.limit_bytes,
            _endian: PhantomData::<fn() -> LittleEndian>,
        };
        let v = T::read(&mut le_reader)?;
        self.total_bytes_read = le_reader.total_bytes_read;
        Ok(v)
    }
    pub fn read_as_le_with<T: ReadWith<U>, U>(&mut self, arg: U) -> Result<T> {
        let mut le_reader: BinaryReader<LittleEndian, &mut R> = BinaryReader {
            reader: &mut self.reader,
            total_bytes_read: self.total_bytes_read,
            limit_bytes: self.limit_bytes,
            _endian: PhantomData::<fn() -> LittleEndian>,
        };
        let v = T::read_with(&mut le_reader, arg)?;
        self.total_bytes_read = le_reader.total_bytes_read;
        Ok(v)
    }
}

// Read trait implementations for primitive types
macro_rules! impl_readable_for_numeric_primitives {
    ( $( $t:ty: $func:ident ),* $(,)? ) => {
        $(
            impl Read for $t {
                #[inline]
                fn read<E: Endian, R: BufRead>(
                    reader: &mut BinaryReader<E, R>
                ) -> Result<Self> {
                    reader.$func()
                }
            }
        )*
    };
}
impl_readable_for_numeric_primitives! {
    u8: u8,
    i8: i8,
    u16: u16,
    i16: i16,
    u32: u32,
    i32: i32,
    f32: f32,
    u64: u64,
    i64: i64,
    f64: f64,
}

impl Read for bool {
    fn read<E: Endian, R: BufRead>(reader: &mut BinaryReader<E, R>) -> Result<Self> {
        match reader.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(ReadError::invalid_data_format("Expected 0 or 1 for boolean")),
        }
    }
}

impl Read for char {
    fn read<E: Endian, R: BufRead>(reader: &mut BinaryReader<E, R>) -> Result<Self> {
        let value = reader.u32()?;
        std::char::from_u32(value).ok_or_else(|| ReadError::invalid_data_format(format!("Invalid char value: {}", value)))
    }
}

macro_rules! impl_readable_for_tuples {
    ( $( ( $( $T:ident ),+ ), )+ ) => {
        $(
            impl<$($T: Read),+> Read for ( $( $T, )+ ) {
                fn read<E: Endian, R: BufRead>(
                    reader: &mut BinaryReader<E, R>
                ) -> Result<Self> {
                    Ok(( $( $T::read(reader)?, )+ ))
                }
            }
        )+
    }
}
impl_readable_for_tuples! {
    (T1, T2),
    (T1, T2, T3),
    (T1, T2, T3, T4),
    (T1, T2, T3, T4, T5),
    (T1, T2, T3, T4, T5, T6),
}

impl<T: Read> ReadWith<bool> for Option<T> {
    fn read_with<E: Endian, R: BufRead>(reader: &mut BinaryReader<E, R>, cond: bool) -> Result<Self> {
        if cond { Ok(Some(T::read(reader)?)) } else { Ok(None) }
    }
}

impl<T: Read + Default, const N: usize> Read for [T; N] {
    fn read<E: Endian, R: BufRead>(reader: &mut BinaryReader<E, R>) -> Result<Self> {
        let mut arr = std::array::from_fn(|_| T::default());
        for e in arr.iter_mut() {
            *e = T::read(reader)?;
        }
        Ok(arr)
    }
}

impl<T: Read> ReadWith<usize> for Vec<T> {
    fn read_with<E: Endian, R: BufRead>(reader: &mut BinaryReader<E, R>, len: usize) -> Result<Self> {
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(T::read(reader)?);
        }
        Ok(vec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integers() {
        let data = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];

        assert_eq!(BinaryReader::from_le_bytes(&data).u8().unwrap(), 0x01);
        assert_eq!(BinaryReader::from_le_bytes(&data).u16().unwrap(), 0x0201);
        assert_eq!(BinaryReader::from_le_bytes(&data).u32().unwrap(), 0x04030201);
        assert_eq!(BinaryReader::from_le_bytes(&data).u64().unwrap(), 0x0807060504030201);

        assert_eq!(BinaryReader::from_le_bytes(&data).i8().unwrap(), 0x01);
        assert_eq!(BinaryReader::from_le_bytes(&data).i16().unwrap(), 0x0201);
        assert_eq!(BinaryReader::from_le_bytes(&data).i32().unwrap(), 0x04030201);
        assert_eq!(BinaryReader::from_le_bytes(&data).i64().unwrap(), 0x0807060504030201);

        assert_eq!(BinaryReader::from_be_bytes(&data).u8().unwrap(), 0x01);
        assert_eq!(BinaryReader::from_be_bytes(&data).u16().unwrap(), 0x0102);
        assert_eq!(BinaryReader::from_be_bytes(&data).u32().unwrap(), 0x01020304);
        assert_eq!(BinaryReader::from_be_bytes(&data).u64().unwrap(), 0x0102030405060708);

        assert_eq!(BinaryReader::from_be_bytes(&data).i8().unwrap(), 0x01);
        assert_eq!(BinaryReader::from_be_bytes(&data).i16().unwrap(), 0x0102);
        assert_eq!(BinaryReader::from_be_bytes(&data).i32().unwrap(), 0x01020304);
        assert_eq!(BinaryReader::from_be_bytes(&data).i64().unwrap(), 0x0102030405060708);
    }

    #[test]
    fn test_floats() {
        let data = [0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x40];
        assert_eq!(BinaryReader::from_le_bytes(&data).f32().unwrap(), 1.0);

        let data = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x3F];
        assert_eq!(BinaryReader::from_le_bytes(&data).f64().unwrap(), 1.0);

        let data = [0x3F, 0x80, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00];
        assert_eq!(BinaryReader::from_be_bytes(&data).f32().unwrap(), 1.0);

        let data = [0x3F, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(BinaryReader::from_be_bytes(&data).f64().unwrap(), 1.0);
    }

    #[test]
    fn test_skip() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut reader = BinaryReader::from_le_bytes(&data);
        assert_eq!(reader.u8().unwrap(), 0x01);
        reader.skip(2).unwrap();
        assert_eq!(reader.u8().unwrap(), 0x04);
    }

    #[test]
    fn test_skip_aligned() {
        let data = [0x01, 0x02, 0x03, 0x04, 0x05];
        let mut reader = BinaryReader::from_le_bytes(&data);
        assert_eq!(reader.u8().unwrap(), 0x01);
        reader.skip_aligned(4).unwrap();
        assert_eq!(reader.u8().unwrap(), 0x05);
    }

    #[test]
    fn test_reserved() {
        let data = [0x01, 0x01, 0x01, 0x01];
        let mut reader = BinaryReader::from_le_bytes(&data);
        reader.reserved::<4>(0x01).unwrap();
    }

    #[test]
    fn test_child() {
        let data = [0x01, 0x02, 0x03, 0x04, 0x05];
        let mut reader = BinaryReader::from_le_bytes(&data);
        reader.value(&0x01u8).unwrap();
        let (v1, v2): (u8, u16) = reader.read_partial(3).unwrap();
        assert_eq!(v1, 0x02);
        assert_eq!(v2, 0x0403);
    }

    #[test]
    fn test_read() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut reader = BinaryReader::from_le_bytes(&data);
        assert_eq!(reader.read::<u8>().unwrap(), 0x01);
        assert_eq!(reader.read::<u16>().unwrap(), 0x302);
        assert_eq!(reader.read::<u8>().unwrap(), 0x04);
    }

    #[test]
    fn test_read_with() {
        struct Test {
            a: u8,
            b: u16,
            opt: Option<u8>,
        }
        impl ReadWith<u8> for Test {
            fn read_with<E: Endian, R: BufRead>(reader: &mut BinaryReader<E, R>, arg: u8) -> Result<Self> {
                let (a, b) = reader.read::<(u8, u16)>()?;
                Ok(Test { a, b, opt: Some(arg) })
            }
        }
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut reader = BinaryReader::from_le_bytes(&data);
        let test: Test = reader.read_with(0x10).unwrap();
        assert_eq!(test.a, 0x01);
        assert_eq!(test.b, 0x302);
        assert_eq!(test.opt, Some(0x10));
    }

    #[test]
    fn test_read_vec() {
        let data = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06];
        let mut reader = BinaryReader::from_le_bytes(&data);
        let vec: Vec<u16> = reader.read_with(3).unwrap();
        assert_eq!(vec, vec![0x0201, 0x0403, 0x0605]);
    }

    #[test]
    fn test_read_if() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut reader = BinaryReader::from_le_bytes(&data);
        let a: Option<u8> = reader.read_with(true).unwrap();
        assert_eq!(a, Some(0x01));
        let b: Option<u16> = reader.read_with(false).unwrap();
        assert_eq!(b, None);
        let c: Option<u16> = reader.read_with(true).unwrap();
        assert_eq!(c, Some(0x0302));
        let _ = reader.u8().unwrap();
        assert!(matches!(reader.check_size(1), Err(ReadError::NotEnoughBytes { .. })));
    }

    #[test]
    fn test_values() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut reader = BinaryReader::from_le_bytes(&data);
        reader.values::<u8>(&[0x01, 0x02]).unwrap();
        reader.values::<u8>(&[0x03, 0x04]).unwrap();
        assert!(matches!(reader.check_size(1), Err(ReadError::NotEnoughBytes { .. })));
    }

    #[test]
    fn test_read_as_be() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut reader = BinaryReader::from_le_bytes(&data);
        let a: u16 = reader.read_as_be().unwrap();
        assert_eq!(a, 0x0102);
        let b: u16 = reader.read().unwrap();
        assert_eq!(b, 0x0403);
        assert!(matches!(reader.check_size(1), Err(ReadError::NotEnoughBytes { .. })));
    }

    #[test]
    fn test_read_as_be_with() {
        struct Test {
            a: u8,
            b: u16,
            opt: Option<u8>,
        }
        impl ReadWith<u8> for Test {
            fn read_with<E: Endian, R: BufRead>(reader: &mut BinaryReader<E, R>, arg: u8) -> Result<Self> {
                let (a, b) = reader.read::<(u8, u16)>()?;
                Ok(Test { a, b, opt: Some(arg) })
            }
        }
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut reader = BinaryReader::from_le_bytes(&data);
        let test: Test = reader.read_as_be_with(0x10).unwrap();
        assert_eq!(test.a, 0x01);
        assert_eq!(test.b, 0x0203);
        assert_eq!(test.opt, Some(0x10));
    }

    #[test]
    fn test_read_tuple() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut reader = BinaryReader::from_le_bytes(&data);
        let (a, b): (u8, u16) = reader.read().unwrap();
        assert_eq!(a, 0x01);
        assert_eq!(b, 0x0302);
    }

    #[test]
    fn test_read_array() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut reader = BinaryReader::from_be_bytes(&data);
        let array: [u16; 2] = reader.read().unwrap();
        assert_eq!(array, [0x0102, 0x0304]);
    }
}
