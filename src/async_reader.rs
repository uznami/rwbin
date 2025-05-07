use super::endian::{BigEndian, Endian, LittleEndian};
use crate::result::{ReadError, ReadResult};
use std::{fmt::Debug, marker::PhantomData};
use tokio::io::AsyncReadExt;

pub struct AsyncBinaryReader<E, R> {
    reader: R,
    total_bytes_read: usize,
    limit_bytes: Option<usize>,
    _endian: PhantomData<fn() -> E>,
}

pub type Result<T> = ReadResult<T>;

pub trait ReaderBase: tokio::io::AsyncBufRead + Unpin + Send {}
impl<T> ReaderBase for T where T: tokio::io::AsyncBufRead + Unpin + Send {}

pub trait AsyncRead: Sized {
    fn read<E: Endian, R: ReaderBase>(reader: &mut AsyncBinaryReader<E, R>) -> impl Future<Output = Result<Self>> + Send;
}

pub trait AsyncReadWith<A>: Sized {
    fn read_with<E: Endian, R: ReaderBase>(reader: &mut AsyncBinaryReader<E, R>, arg: A) -> impl Future<Output = Result<Self>> + Send;
}

impl<'a> AsyncBinaryReader<BigEndian, tokio::io::BufReader<&'a [u8]>> {
    pub fn from_be_bytes(buf: &'a [u8]) -> Self {
        let reader = tokio::io::BufReader::new(buf);
        AsyncBinaryReader {
            reader,
            total_bytes_read: 0,
            limit_bytes: Some(buf.len()),
            _endian: PhantomData::<fn() -> BigEndian>,
        }
    }
}

impl<R: ReaderBase> AsyncBinaryReader<BigEndian, R> {
    pub fn new_be(reader: R) -> Self {
        AsyncBinaryReader {
            reader,
            total_bytes_read: 0,
            limit_bytes: None,
            _endian: PhantomData::<fn() -> BigEndian>,
        }
    }
    pub async fn read_as_le<T: AsyncRead>(&mut self) -> Result<T> {
        let mut rdr = AsyncBinaryReader {
            reader: &mut self.reader,
            total_bytes_read: self.total_bytes_read,
            limit_bytes: self.limit_bytes,
            _endian: PhantomData::<fn() -> LittleEndian>,
        };
        let res = T::read(&mut rdr).await?;
        self.total_bytes_read = rdr.total_bytes_read; // take over the total_bytes_read
        Ok(res)
    }
    pub async fn read_as_le_with<A, T: AsyncReadWith<A>>(&mut self, arg: A) -> Result<T> {
        let mut rdr = AsyncBinaryReader {
            reader: &mut self.reader,
            total_bytes_read: self.total_bytes_read,
            limit_bytes: self.limit_bytes,
            _endian: PhantomData::<fn() -> LittleEndian>,
        };
        let res = T::read_with(&mut rdr, arg).await?;
        self.total_bytes_read = rdr.total_bytes_read; // take over the total_bytes_read
        Ok(res)
    }
}

impl<'a> AsyncBinaryReader<LittleEndian, tokio::io::BufReader<&'a [u8]>> {
    pub fn from_le_bytes(buf: &'a [u8]) -> Self {
        let reader = tokio::io::BufReader::new(buf);
        AsyncBinaryReader {
            reader,
            total_bytes_read: 0,
            limit_bytes: Some(buf.len()),
            _endian: PhantomData,
        }
    }
}

impl<R: ReaderBase> AsyncBinaryReader<LittleEndian, R> {
    pub fn new_le(reader: R) -> Self {
        AsyncBinaryReader {
            reader,
            total_bytes_read: 0,
            limit_bytes: None,
            _endian: PhantomData,
        }
    }
    pub async fn read_as_be<T: AsyncRead>(&mut self) -> Result<T> {
        let mut rdr = AsyncBinaryReader {
            reader: &mut self.reader,
            total_bytes_read: self.total_bytes_read,
            limit_bytes: self.limit_bytes,
            _endian: PhantomData::<fn() -> BigEndian>,
        };
        let res = T::read(&mut rdr).await?;
        self.total_bytes_read = rdr.total_bytes_read; // take over the total_bytes_read
        Ok(res)
    }
    pub async fn read_as_be_with<A, T: AsyncReadWith<A>>(&mut self, arg: A) -> Result<T> {
        let mut rdr = AsyncBinaryReader {
            reader: &mut self.reader,
            total_bytes_read: self.total_bytes_read,
            limit_bytes: self.limit_bytes,
            _endian: PhantomData::<fn() -> BigEndian>,
        };
        let res = T::read_with(&mut rdr, arg).await?;
        self.total_bytes_read = rdr.total_bytes_read; // take over the total_bytes_read
        Ok(res)
    }
}

impl<E: Endian, R: ReaderBase> AsyncBinaryReader<E, R> {
    #[inline]
    fn check_size(&self, len: usize) -> Result<()> {
        if let Some(limit) = self.limit_bytes {
            if self.total_bytes_read + len > limit {
                return Err(ReadError::not_enough_bytes(len, limit - self.total_bytes_read));
            }
        }
        Ok(())
    }
    #[inline]
    pub async fn read_from_slice<T>(&mut self, len: usize, parse: impl Fn(&[u8]) -> Result<T>) -> Result<T> {
        self.check_size(len)?;
        const POPULAR_BUF_SIZE: usize = 512;
        if len <= POPULAR_BUF_SIZE {
            let mut buf = [0u8; POPULAR_BUF_SIZE];
            self.reader.read_exact(&mut buf[..len]).await.map_err(ReadError::io_error)?;
            self.total_bytes_read += len;
            parse(&buf[..len])
        } else {
            let mut buf = vec![0u8; len];
            self.reader.read_exact(&mut buf).await.map_err(ReadError::io_error)?;
            self.total_bytes_read += len;
            parse(&buf)
        }
    }

    pub async fn read_from_array<T, const N: usize>(&mut self, parse: impl Fn(&[u8; N]) -> T) -> Result<T> {
        self.check_size(N)?;
        let mut buf = [0u8; N];
        self.reader.read_exact(&mut buf).await.map_err(ReadError::io_error)?;
        self.total_bytes_read += N;
        Ok(parse(&buf))
    }

    #[inline]
    pub async fn read_while<T, const N: usize>(&mut self, try_parse: impl Fn(&[u8; N]) -> Option<T>) -> Result<Vec<T>> {
        let mut values = Vec::new();
        let mut buf = [0u8; N];
        loop {
            self.check_size(N)?;
            self.reader.read_exact(&mut buf).await.map_err(ReadError::io_error)?;
            self.total_bytes_read += N;
            match try_parse(&buf) {
                Some(value) => values.push(value),
                None => break,
            }
        }
        Ok(values)
    }
    #[inline]
    pub async fn u8(&mut self) -> Result<u8> {
        self.read_from_array(|b: &[u8; 1]| b[0]).await
    }
    #[inline]
    pub async fn i8(&mut self) -> Result<i8> {
        self.read_from_array(|b: &[u8; 1]| b[0] as i8).await
    }
    #[inline]
    pub async fn u16(&mut self) -> Result<u16> {
        self.read_from_array(E::u16_from_bytes).await
    }
    #[inline]
    pub async fn i16(&mut self) -> Result<i16> {
        self.read_from_array(E::i16_from_bytes).await
    }
    #[inline]
    pub async fn u32(&mut self) -> Result<u32> {
        self.read_from_array(E::u32_from_bytes).await
    }
    #[inline]
    pub async fn i32(&mut self) -> Result<i32> {
        self.read_from_array(E::i32_from_bytes).await
    }
    #[inline]
    pub async fn f32(&mut self) -> Result<f32> {
        self.read_from_array(E::f32_from_bytes).await
    }
    #[inline]
    pub async fn u64(&mut self) -> Result<u64> {
        self.read_from_array(E::u64_from_bytes).await
    }
    #[inline]
    pub async fn i64(&mut self) -> Result<i64> {
        self.read_from_array(E::i64_from_bytes).await
    }
    #[inline]
    pub async fn f64(&mut self) -> Result<f64> {
        self.read_from_array(E::f64_from_bytes).await
    }

    #[inline]
    pub async fn read<T: AsyncRead>(&mut self) -> Result<T> {
        T::read(self).await
    }
    #[inline]
    pub async fn read_with<A, T: AsyncReadWith<A>>(&mut self, arg: A) -> Result<T> {
        T::read_with(self, arg).await
    }

    #[inline]
    pub async fn value<T: AsyncRead + PartialEq + Debug>(&mut self, expected: &T) -> Result<()> {
        let actual = self.read::<T>().await?;
        if actual == *expected {
            Ok(())
        } else {
            Err(ReadError::invalid_data_format(format!("Expected {:?}, got {:?}", expected, actual)))
        }
    }

    #[inline]
    pub async fn values<T: AsyncRead + PartialEq + Debug>(&mut self, expected: &[T]) -> Result<()> {
        for e in expected {
            self.value(e).await?;
        }
        Ok(())
    }

    #[inline]
    pub async fn reserved<const N: usize>(&mut self, expected_value: u8) -> Result<()> {
        self.read_from_array(|buf: &[u8; N]| {
            for &byte in buf {
                if byte != expected_value {
                    return Err(ReadError::invalid_data_format(format!("Expected {:?}, got {:?}", expected_value, byte)));
                }
            }
            Ok(())
        })
        .await?
    }

    #[inline]
    pub async fn skip(&mut self, len: usize) -> Result<()> {
        self.read_from_slice(len, |_| Ok(())).await
    }

    #[inline]
    pub async fn skip_aligned(&mut self, align: usize) -> Result<()> {
        let remainder = self.total_bytes_read % align;
        if remainder != 0 {
            self.skip(align - remainder).await?;
        }
        Ok(())
    }

    #[inline]
    pub async fn read_partial<T: AsyncRead>(&mut self, len: usize) -> Result<T> {
        let original_limit = self.limit_bytes; // Save the original limit
        self.limit_bytes = Some(len + self.total_bytes_read);
        let result = T::read(self).await?;
        self.limit_bytes = original_limit; // Restore the original limit
        Ok(result)
    }

    #[inline]
    pub async fn read_partial_with<A, T: AsyncReadWith<A>>(&mut self, len: usize, arg: A) -> Result<T> {
        let original_limit = self.limit_bytes; // Save the original limit
        self.limit_bytes = Some(len + self.total_bytes_read);
        let result = T::read_with(self, arg).await?;
        self.limit_bytes = original_limit; // Restore the original limit
        Ok(result)
    }
}

macro_rules! impl_readable_for_numeric_primitives {
    ( $( $t:ty: $func:ident ),* $(,)? ) => {
        $(
            impl AsyncRead for $t {
                #[inline]
                async fn read<E: Endian, R: ReaderBase>(
                    reader: &mut AsyncBinaryReader<E, R>
                ) -> Result<Self> {
                    reader.$func().await
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

impl AsyncRead for bool {
    #[inline]
    async fn read<E: Endian, R: ReaderBase>(reader: &mut AsyncBinaryReader<E, R>) -> Result<Self> {
        match reader.u8().await? {
            0 => Ok(false),
            1 => Ok(true),
            v => Err(ReadError::invalid_data_format(format!("Invalid boolean value: {}", v))),
        }
    }
}

impl AsyncRead for char {
    #[inline]
    async fn read<E: Endian, R: ReaderBase>(reader: &mut AsyncBinaryReader<E, R>) -> Result<Self> {
        let ch = reader.u32().await?;
        char::from_u32(ch).ok_or_else(|| ReadError::invalid_data_format(format!("Invalid char value: {}", ch)))
    }
}

impl<T: AsyncRead + Send> AsyncReadWith<bool> for Option<T> {
    #[inline]
    async fn read_with<E: Endian, R: ReaderBase>(reader: &mut AsyncBinaryReader<E, R>, cond: bool) -> Result<Self> {
        if cond { Ok(Some(T::read(reader).await?)) } else { Ok(None) }
    }
}

macro_rules! impl_readable_for_tuples {
    ( $( ( $( $T:ident ),+ ), )+ ) => {
        $(
            impl<$( $T: AsyncRead + Send ),+> AsyncRead for ( $( $T, )+ ) {
                #[inline]
                async fn read<E: Endian, R: ReaderBase>(
                    reader: &mut AsyncBinaryReader<E, R>
                ) -> Result<Self> {
                    Ok(( $( $T::read(reader).await?, )+ ))
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

impl<T: AsyncRead + Send + Default, const N: usize> AsyncRead for [T; N] {
    #[inline]
    async fn read<E: Endian, R: ReaderBase>(reader: &mut AsyncBinaryReader<E, R>) -> Result<Self> {
        let mut arr = std::array::from_fn(|_| T::default());
        for elem in arr.iter_mut() {
            *elem = T::read(reader).await?;
        }
        Ok(arr)
    }
}

impl<T: AsyncRead + Send> AsyncReadWith<usize> for Vec<T> {
    #[inline]
    async fn read_with<E: Endian, R: ReaderBase>(reader: &mut AsyncBinaryReader<E, R>, len: usize) -> Result<Self> {
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(T::read(reader).await?);
        }
        Ok(vec)
    }
}

impl<A: Send + Sync, T: for<'a> AsyncReadWith<&'a A> + Send> AsyncReadWith<(usize, &A)> for Vec<T> {
    #[inline]
    async fn read_with<E: Endian, R: ReaderBase>(reader: &mut AsyncBinaryReader<E, R>, (len, arg): (usize, &A)) -> Result<Self> {
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(T::read_with(reader, arg).await?);
        }
        Ok(vec)
    }
}

#[tokio::test]
async fn test_reserverd() {
    let buf: [u8; 4] = [0, 0, 0, 0];
    let mut reader = AsyncBinaryReader::from_be_bytes(&buf);
    reader.reserved::<4>(0x00).await.unwrap();
}
