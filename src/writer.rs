use crate::{
    endian::{BigEndian, Endian, LittleEndian},
    result::{WriteError, WriteResult},
};
use std::marker::PhantomData;

pub struct BinaryWriter<E, W> {
    writer: W,
    _endian: PhantomData<fn() -> E>,
}

pub type Result<T> = WriteResult<T>;

impl<W: std::io::Write> BinaryWriter<LittleEndian, W> {
    /// Creates a new little-endian BinaryWriter wrapping the given writer.
    pub fn new_le(writer: W) -> Self {
        BinaryWriter { writer, _endian: PhantomData }
    }
}

impl<W: std::io::Write> BinaryWriter<BigEndian, W> {
    /// Creates a new big-endian BinaryWriter wrapping the given writer.
    pub fn new_be(writer: W) -> Self {
        BinaryWriter { writer, _endian: PhantomData }
    }
}

impl<W: std::io::Write, E: Endian> BinaryWriter<E, W> {
    /// Writes raw bytes.
    #[inline]
    fn write_value<const N: usize>(&mut self, value: &[u8; N]) -> Result<()> {
        self.writer.write_all(value).map_err(WriteError::io_error)?;
        Ok(())
    }

    /// Writes an unsigned 8‑bit integer.
    #[inline]
    pub fn u8(&mut self, value: u8) -> Result<()> {
        self.write_value(&[value])
    }

    /// Writes a signed 8‑bit integer.
    #[inline]
    pub fn i8(&mut self, value: i8) -> Result<()> {
        self.write_value(&[value as u8])
    }

    /// Writes an unsigned 16‑bit integer according to the configured endian.
    #[inline]
    pub fn u16(&mut self, value: u16) -> Result<()> {
        self.write_value(&E::u16_to_bytes(value))
    }

    /// Writes a signed 16‑bit integer according to the configured endian.
    #[inline]
    pub fn i16(&mut self, value: i16) -> Result<()> {
        self.write_value(&E::i16_to_bytes(value))
    }

    /// Writes an unsigned 32‑bit integer according to the configured endian.
    #[inline]
    pub fn u32(&mut self, value: u32) -> Result<()> {
        self.write_value(&E::u32_to_bytes(value))
    }

    /// Writes a signed 32‑bit integer according to the configured endian.
    #[inline]
    pub fn i32(&mut self, value: i32) -> Result<()> {
        self.write_value(&E::i32_to_bytes(value))
    }

    /// Writes a 32‑bit floating point value according to the configured endian.
    #[inline]
    pub fn f32(&mut self, value: f32) -> Result<()> {
        self.write_value(&E::f32_to_bytes(value))
    }

    /// Writes an unsigned 64‑bit integer according to the configured endian.
    #[inline]
    pub fn u64(&mut self, value: u64) -> Result<()> {
        self.write_value(&E::u64_to_bytes(value))
    }

    /// Writes a signed 64‑bit integer according to the configured endian.
    #[inline]
    pub fn i64(&mut self, value: i64) -> Result<()> {
        self.write_value(&E::i64_to_bytes(value))
    }

    /// Writes a 64‑bit floating point value according to the configured endian.
    #[inline]
    pub fn f64(&mut self, value: f64) -> Result<()> {
        self.write_value(&E::f64_to_bytes(value))
    }

    /// Writes `len` bytes of the given value (reserved space).
    #[inline]
    pub fn reserved(&mut self, value: u8, len: usize) -> Result<()> {
        self.writer.write_all(&vec![value; len]).map_err(WriteError::io_error)?;
        Ok(())
    }

    /// Pads output with zeros up to the next multiple of `alignment`.
    #[inline]
    pub fn fill_aligned(&mut self, alignment: usize, offset: usize) -> Result<()> {
        let remainder = alignment - (offset % alignment);
        if remainder == 0 {
            return Ok(());
        }
        self.reserved(0x00, remainder)
    }

    /// Flushes the underlying writer.
    #[inline]
    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush().map_err(WriteError::io_error)
    }
}

pub trait Write {
    fn write<E: Endian, W: std::io::Write>(&self, writer: &mut BinaryWriter<E, W>) -> Result<()>;
}

pub trait WriteWith<T> {
    fn write_with<E: Endian, W: std::io::Write>(&self, writer: &mut BinaryWriter<E, W>, value: T) -> Result<()>;
}

impl<W: std::io::Write> BinaryWriter<LittleEndian, W> {
    /// Writes the given value as big-endian without changing this writer’s endian.
    pub fn write_as_be<T: Write>(&mut self, value: &T) -> Result<()> {
        let mut writer = BinaryWriter {
            writer: &mut self.writer,
            _endian: PhantomData::<fn() -> BigEndian>,
        };
        writer.write(value)?;
        Ok(())
    }

    /// Writes the given value with parameter as big-endian.
    pub fn write_as_be_with<T: WriteWith<U>, U>(&mut self, value: &T, with: U) -> Result<()> {
        let mut writer = BinaryWriter {
            writer: &mut self.writer,
            _endian: PhantomData::<fn() -> BigEndian>,
        };
        value.write_with(&mut writer, with)?;
        Ok(())
    }
}

impl<W: std::io::Write> BinaryWriter<BigEndian, W> {
    /// Writes the given value as little-endian without changing this writer’s endian.
    pub fn write_as_le<T: Write>(&mut self, value: &T) -> Result<()> {
        let mut writer = BinaryWriter {
            writer: &mut self.writer,
            _endian: PhantomData::<fn() -> LittleEndian>,
        };
        writer.write(value)?;
        Ok(())
    }

    /// Writes the given value with parameter as little-endian.
    pub fn write_as_le_with<T: WriteWith<U>, U>(&mut self, value: &T, with: U) -> Result<()> {
        let mut writer = BinaryWriter {
            writer: &mut self.writer,
            _endian: PhantomData::<fn() -> LittleEndian>,
        };
        value.write_with(&mut writer, with)?;
        Ok(())
    }
}

impl<W: std::io::Write, E: Endian> BinaryWriter<E, W> {
    /// Writes any value implementing the `Write` trait.
    #[inline]
    pub fn write<T: Write>(&mut self, value: &T) -> Result<()> {
        value.write(self)
    }

    /// Writes any value implementing the `WriteWith` trait with a parameter.
    #[inline]
    pub fn write_with<T, U>(&mut self, value: &T, with: U) -> Result<()>
    where
        T: WriteWith<U>,
    {
        value.write_with(self, with)
    }
}

macro_rules! impl_binary_writable {
    ($($ty:ty => $func:ident),* $(,)?) => {
        $(
            impl Write for $ty {
                fn write<E: Endian, W: std::io::Write>(
                    &self,
                    writer: &mut BinaryWriter<E, W>,
                ) -> Result<()> {
                    writer.$func(*self)?;
                    Ok(())
                }
            }
        )*
    };
}

impl_binary_writable! {
    u8 => u8,
    i8 => i8,
    u16 => u16,
    i16 => i16,
    u32 => u32,
    i32 => i32,
    f32 => f32,
    u64 => u64,
    i64 => i64,
    f64 => f64,
}

impl Write for bool {
    fn write<E: Endian, W: std::io::Write>(&self, writer: &mut BinaryWriter<E, W>) -> Result<()> {
        let value = if *self { 1u8 } else { 0u8 };
        writer.u8(value)
    }
}

impl Write for char {
    fn write<E: Endian, W: std::io::Write>(&self, writer: &mut BinaryWriter<E, W>) -> Result<()> {
        writer.u32(*self as u32)
    }
}

impl<T: Write> Write for Option<T> {
    fn write<E: Endian, W: std::io::Write>(&self, writer: &mut BinaryWriter<E, W>) -> Result<()> {
        if let Some(value) = self {
            writer.write(value)?;
        }
        Ok(())
    }
}

impl<T: Write> Write for &[T] {
    fn write<E: Endian, W: std::io::Write>(&self, writer: &mut BinaryWriter<E, W>) -> Result<()> {
        if std::mem::size_of::<T>() == 1 {
            // SAFETY: We are assuming that T is a byte, so this is safe
            let bytes = unsafe { std::slice::from_raw_parts(self.as_ptr() as *const u8, self.len()) };
            writer.writer.write_all(bytes).map_err(WriteError::io_error)?;
            Ok(())
        } else {
            for item in self.iter() {
                writer.write(item)?;
            }
            Ok(())
        }
    }
}

impl<T: Write, const N: usize> Write for [T; N] {
    fn write<E: Endian, W: std::io::Write>(&self, writer: &mut BinaryWriter<E, W>) -> Result<()> {
        writer.write(&self.as_slice())
    }
}

impl<T: Write> Write for Vec<T> {
    fn write<E: Endian, W: std::io::Write>(&self, writer: &mut BinaryWriter<E, W>) -> Result<()> {
        writer.write(&self.as_slice())
    }
}

macro_rules! impl_writable_for_tuples {
    ( $( ( $( $T:ident ),+ ), )+ ) => {
        $(
            #[allow(non_snake_case)]
            impl<$($T: Write),+> Write for ( $( $T, )+ ) {
                fn write<E: Endian, W: std::io::Write>(
                    &self,
                    writer: &mut BinaryWriter<E, W>,
                ) -> Result<()> {
                    let ($(ref $T,)+) = *self;
                    $(
                        $T.write(writer)?;
                    )+
                    Ok(())
                }
            }
        )+
    }
}
impl_writable_for_tuples! {
    (T1, T2),
    (T1, T2, T3),
    (T1, T2, T3, T4),
    (T1, T2, T3, T4, T5),
    (T1, T2, T3, T4, T5, T6),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_writer() {
        let mut buf = Vec::new();
        let mut writer = BinaryWriter::new_le(&mut buf);

        writer.write(&0x01u8).unwrap();
        writer.write(&0x0203i16).unwrap();
        writer.write(&0x04050607u32).unwrap();
        writer.write(&0x08090a0b0c0d0e0fu64).unwrap();

        assert_eq!(buf, vec![0x01, 0x03, 0x02, 0x07, 0x06, 0x05, 0x04, 0x0f, 0x0e, 0x0d, 0x0c, 0x0b, 0x0a, 0x09, 0x08]);
    }

    #[test]
    fn test_immediate_write() {
        let mut buf = Vec::new();
        let mut writer = BinaryWriter::new_le(&mut buf);

        writer.write_as_be(&0x0102_u16).unwrap();
        writer.write(&0x0304_u16).unwrap();

        assert_eq!(buf, vec![0x01, 0x02, 0x04, 0x03]);
    }

    #[test]
    fn test_binary_writer_tuple() {
        let mut buf = Vec::new();
        let mut writer = BinaryWriter::new_le(&mut buf);

        writer.write(&(0x_01_u8, 0x_0203_i16)).unwrap();
        writer.write(&(0x_0405_0607_u32, 0x_0809_0a0b_0c0d_0e0f_u64)).unwrap();

        writer.write(&vec![0x_0001_u16, 0x_0203_u16]).unwrap();

        assert_eq!(
            buf,
            vec![0x01, 0x03, 0x02, 0x07, 0x06, 0x05, 0x04, 0x0f, 0x0e, 0x0d, 0x0c, 0x0b, 0x0a, 0x09, 0x08, 0x01, 0x00, 0x03, 0x02]
        );
    }
}
