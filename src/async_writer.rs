use super::endian::{BigEndian, Endian, LittleEndian};
use crate::result::{WriteError, WriteResult};
use std::marker::PhantomData;
use tokio::io::AsyncWriteExt;

pub struct AsyncBinaryWriter<E, R> {
    writer: R,
    _endian: PhantomData<fn() -> E>,
}

pub type Result<T> = WriteResult<T>;

pub trait WriterBase: tokio::io::AsyncWrite + Unpin + Send {}
impl<T> WriterBase for T where T: tokio::io::AsyncWrite + Unpin + Send {}

impl<W: WriterBase> AsyncBinaryWriter<BigEndian, W> {
    pub fn new_be(writer: W) -> Self {
        AsyncBinaryWriter { writer, _endian: PhantomData }
    }
}

impl<W: WriterBase> AsyncBinaryWriter<LittleEndian, W> {
    pub fn new_le(writer: W) -> Self {
        AsyncBinaryWriter { writer, _endian: PhantomData }
    }
}

impl<E: Endian, W: WriterBase> AsyncBinaryWriter<E, W> {
    #[inline]
    async fn write_value<const N: usize>(&mut self, value: &[u8; N]) -> Result<()> {
        self.writer.write_all(value).await.map_err(WriteError::io_error)
    }
    #[inline]
    pub async fn u8(&mut self, value: u8) -> Result<()> {
        self.write_value(&[value]).await
    }
    #[inline]
    pub async fn i8(&mut self, value: i8) -> Result<()> {
        self.write_value(&[value as u8]).await
    }
    #[inline]
    pub async fn u16(&mut self, value: u16) -> Result<()> {
        self.write_value(&E::u16_to_bytes(value)).await
    }
    #[inline]
    pub async fn i16(&mut self, value: i16) -> Result<()> {
        self.write_value(&E::i16_to_bytes(value)).await
    }
    #[inline]
    pub async fn u32(&mut self, value: u32) -> Result<()> {
        self.write_value(&E::u32_to_bytes(value)).await
    }
    #[inline]
    pub async fn i32(&mut self, value: i32) -> Result<()> {
        self.write_value(&E::i32_to_bytes(value)).await
    }
    #[inline]
    pub async fn f32(&mut self, value: f32) -> Result<()> {
        self.write_value(&E::f32_to_bytes(value)).await
    }
    #[inline]
    pub async fn u64(&mut self, value: u64) -> Result<()> {
        self.write_value(&E::u64_to_bytes(value)).await
    }
    #[inline]
    pub async fn i64(&mut self, value: i64) -> Result<()> {
        self.write_value(&E::i64_to_bytes(value)).await
    }
    #[inline]
    pub async fn f64(&mut self, value: f64) -> Result<()> {
        self.write_value(&E::f64_to_bytes(value)).await
    }
    #[inline]
    pub async fn reserved(&mut self, value: u8, length: usize) -> Result<()> {
        const POPULAR_BUF_SIZE: usize = 512;
        if length <= POPULAR_BUF_SIZE {
            let buf = [value; POPULAR_BUF_SIZE];
            self.writer.write_all(&buf[..length]).await.map_err(WriteError::io_error)?;
        } else {
            let buf = vec![value; length];
            self.writer.write_all(&buf).await.map_err(WriteError::io_error)?;
        }
        Ok(())
    }
    #[inline]
    pub async fn fill_aligned(&mut self, alignment: usize, offset: usize) -> Result<()> {
        let remainder = alignment - (offset % alignment);
        if remainder == 0 {
            return Ok(());
        }
        self.reserved(0x00, remainder).await
    }
    #[inline]
    pub async fn flush(&mut self) -> Result<()> {
        self.writer.flush().await.map_err(WriteError::io_error)
    }
}

pub trait AsyncWrite {
    fn write<E: Endian, W: WriterBase>(&self, writer: &mut AsyncBinaryWriter<E, W>) -> impl Future<Output = Result<()>> + Send;
}

pub trait AsyncWriteWith<T> {
    fn write_with<E: Endian, W: WriterBase>(&self, writer: &mut AsyncBinaryWriter<E, W>, arg: &T) -> impl Future<Output = Result<()>> + Send;
}

impl<W: WriterBase> AsyncBinaryWriter<BigEndian, W> {
    pub async fn write_as_le<T: AsyncWrite>(&mut self, value: &T) -> Result<()> {
        let mut writer = AsyncBinaryWriter {
            writer: &mut self.writer,
            _endian: PhantomData::<fn() -> LittleEndian>,
        };
        value.write(&mut writer).await?;
        Ok(())
    }
    pub async fn write_as_le_with<U, T: AsyncWriteWith<U>>(&mut self, value: T, with: U) -> Result<()> {
        let mut writer = AsyncBinaryWriter {
            writer: &mut self.writer,
            _endian: PhantomData::<fn() -> LittleEndian>,
        };
        value.write_with(&mut writer, &with).await?;
        Ok(())
    }
}

impl<W: WriterBase> AsyncBinaryWriter<LittleEndian, W> {
    pub async fn write_as_be<T: AsyncWrite>(&mut self, value: &T) -> Result<()> {
        let mut writer = AsyncBinaryWriter {
            writer: &mut self.writer,
            _endian: PhantomData::<fn() -> BigEndian>,
        };
        value.write(&mut writer).await?;
        Ok(())
    }
    pub async fn write_as_be_with<T: AsyncWriteWith<U>, U>(&mut self, value: &T, with: U) -> Result<()> {
        let mut writer = AsyncBinaryWriter {
            writer: &mut self.writer,
            _endian: PhantomData::<fn() -> BigEndian>,
        };
        value.write_with(&mut writer, &with).await?;
        Ok(())
    }
}

impl<E: Endian, W: WriterBase> AsyncBinaryWriter<E, W> {
    #[inline]
    pub async fn write<T: AsyncWrite>(&mut self, value: &T) -> Result<()> {
        value.write(self).await
    }
    #[inline]
    pub async fn write_with<A, T: AsyncWriteWith<A> + ?Sized>(&mut self, value: &T, arg: &A) -> Result<()> {
        value.write_with(self, arg).await
    }
}

macro_rules! impl_writable_for_numeric_primitives {
    ($($t:ty: $method:ident),* $(,)?) => {
        $(
            impl AsyncWrite for $t {
                async fn write<E: Endian, W: WriterBase>(&self, writer: &mut AsyncBinaryWriter<E, W>) -> Result<()> {
                    writer.$method(*self).await
                }
            }
        )*
    };
}
impl_writable_for_numeric_primitives!(
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
);

impl AsyncWrite for bool {
    async fn write<E: Endian, W: WriterBase>(&self, writer: &mut AsyncBinaryWriter<E, W>) -> Result<()> {
        writer.u8(if *self { 1 } else { 0 }).await
    }
}
impl AsyncWrite for char {
    async fn write<E: Endian, W: WriterBase>(&self, writer: &mut AsyncBinaryWriter<E, W>) -> Result<()> {
        writer.u32(*self as u32).await
    }
}

impl<T: AsyncWrite + Send + Sync> AsyncWrite for Option<T> {
    async fn write<E: Endian, W: WriterBase>(&self, writer: &mut AsyncBinaryWriter<E, W>) -> Result<()> {
        if let Some(value) = self { value.write(writer).await } else { Ok(()) }
    }
}

macro_rules! impl_writable_for_tuple {
    ($($name:ident),+) => {
        #[allow(non_snake_case)]
        impl<$($name: AsyncWrite + Send + Sync),+> AsyncWrite for ($($name,)+) {
            async fn write<E: Endian, W: WriterBase>(&self, writer: &mut AsyncBinaryWriter<E, W>) -> Result<()> {
                let ($(ref $name,)+) = *self;
                $(
                    $name.write(writer).await?;
                )+
                Ok(())
            }
        }
    };
}
impl_writable_for_tuple!(T1, T2);
impl_writable_for_tuple!(T1, T2, T3);
impl_writable_for_tuple!(T1, T2, T3, T4);
impl_writable_for_tuple!(T1, T2, T3, T4, T5);
impl_writable_for_tuple!(T1, T2, T3, T4, T5, T6);

impl<T: AsyncWrite + Send + Sync> AsyncWrite for &[T] {
    async fn write<E: Endian, W: WriterBase>(&self, writer: &mut AsyncBinaryWriter<E, W>) -> Result<()> {
        if std::mem::size_of::<T>() == 1 {
            // SAFETY: We are assuming that T is a byte, so this is safe
            let bytes = unsafe { std::slice::from_raw_parts(self.as_ptr() as *const u8, self.len()) };
            writer.writer.write_all(bytes).await.map_err(WriteError::io_error)?;
        } else {
            for item in self.iter() {
                item.write(writer).await?;
            }
        }
        Ok(())
    }
}

impl<T: AsyncWrite + Send + Sync, const N: usize> AsyncWrite for [T; N] {
    async fn write<E: Endian, W: WriterBase>(&self, writer: &mut AsyncBinaryWriter<E, W>) -> Result<()> {
        (&self[..]).write(writer).await
    }
}

impl<T: AsyncWrite + Send + Sync> AsyncWrite for Vec<T> {
    async fn write<E: Endian, W: WriterBase>(&self, writer: &mut AsyncBinaryWriter<E, W>) -> Result<()> {
        self.as_slice().write(writer).await
    }
}

#[tokio::test]
async fn test_async_binary_writer() {
    use std::io::Cursor;
    let mut data = vec![0; 16];
    let stream = tokio::io::BufStream::new(Cursor::new(&mut data));
    let mut writer = AsyncBinaryWriter::new_be(stream);

    assert!(writer.write(&29u8).await.is_ok());
    assert!(writer.fill_aligned(4, 1).await.is_ok());
    assert!(writer.write(&[1u8, 2, 3, 4]).await.is_ok());
    assert!(writer.reserved(0, 1).await.is_ok());
    assert!(writer.write(&(0x56_u8, 0x789A_u16)).await.is_ok());
    assert!(writer.write(&0x12345678_u32).await.is_ok());
    assert!(writer.flush().await.is_ok());

    // check the written data
    let expected = [
        29, 0, 0, 0, // 4-byte alignment
        1, 2, 3, 4, // 4 bytes
        0, // 1 byte reserved
        0x56, 0x78, 0x9A, // 3 bytes for u8 and u16
        0x12, 0x34, 0x56, 0x78, // 4 bytes
    ];
    assert_eq!(&data[..], &expected[..]);
}
