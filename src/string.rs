use crate::{async_reader::AsyncBinaryReader, async_writer::AsyncBinaryWriter, endian::Endian, reader::BinaryReader, writer::BinaryWriter};

pub enum StringMode {
    FixedChars(usize),
    NullTerminated,
}

#[test]
fn hoge() {
    let data = b"Hello, world!\0\0\0";
    let len = data.iter().position(|&c| c == 0).unwrap_or(data.len());
    let parsed = String::from_utf8_lossy(&data[..len]).to_string();
    println!("{:?}", parsed);
}

fn parse_fixed_utf8(data: &[u8]) -> String {
    let len = data.iter().position(|&c| c == 0).unwrap_or(data.len());
    String::from_utf8_lossy(&data[..len]).to_string()
}
fn parse_fixed_utf16<E: Endian>(data: &[u8]) -> String {
    let data = E::u16vec_from_bytes(data);
    let len = data.iter().position(|&c| c == 0).unwrap_or(data.len());
    String::from_utf16_lossy(&data[..len])
}
fn take_utf8_char(data: &[u8; 1]) -> Option<u8> {
    match data[0] {
        0 => None,
        c => Some(c),
    }
}
fn take_u16_char<E: Endian>(data: &[u8; 2]) -> Option<u16> {
    match E::u16_from_bytes(data) {
        0 => None,
        c => Some(c),
    }
}

impl<E: Endian, R: std::io::BufRead> BinaryReader<E, R> {
    pub fn utf8_str(&mut self, mode: StringMode) -> crate::reader::Result<String> {
        match mode {
            StringMode::FixedChars(num_chars) => self.read_from_slice(num_chars, |data| Ok(parse_fixed_utf8(data))),
            StringMode::NullTerminated => {
                let buf = self.read_while(take_utf8_char)?;
                Ok(String::from_utf8_lossy(&buf).to_string())
            }
        }
    }

    pub fn utf16_str(&mut self, mode: StringMode) -> crate::reader::Result<String> {
        match mode {
            StringMode::FixedChars(num_chars) => self.read_from_slice(2 * num_chars, |data| Ok(parse_fixed_utf16::<E>(data))),
            StringMode::NullTerminated => {
                let buf = self.read_while(take_u16_char::<E>)?;
                Ok(String::from_utf16_lossy(&buf))
            }
        }
    }
}

impl<E: Endian, R: crate::async_reader::ReaderBase> AsyncBinaryReader<E, R> {
    pub async fn utf8_str(&mut self, mode: StringMode) -> crate::async_reader::Result<String> {
        match mode {
            StringMode::FixedChars(num_chars) => self.read_from_slice(num_chars, |data| Ok(parse_fixed_utf8(data))).await,
            StringMode::NullTerminated => {
                let buf: Vec<u8> = self.read_while(take_utf8_char).await?;
                Ok(String::from_utf8_lossy(&buf).to_string())
            }
        }
    }

    pub async fn utf16_str(&mut self, mode: StringMode) -> crate::async_reader::Result<String> {
        match mode {
            StringMode::FixedChars(num_chars) => self.read_from_slice(2 * num_chars, |data| Ok(parse_fixed_utf16::<E>(data))).await,
            StringMode::NullTerminated => {
                let buf: Vec<u16> = self.read_while(take_u16_char::<E>).await?;
                Ok(String::from_utf16_lossy(&buf))
            }
        }
    }
}

impl<W: std::io::Write, E: Endian> BinaryWriter<E, W> {
    pub fn utf8_str<T: AsRef<str>>(&mut self, value: T, mode: StringMode) -> crate::writer::Result<()> {
        let bytes = value.as_ref().as_bytes();
        match mode {
            StringMode::FixedChars(size) => {
                if bytes.len() > size {
                    return Err(crate::result::WriteError::io_error(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "String is too long for fixed size",
                    )));
                }
                let padding = size - bytes.len();
                self.write(&bytes)?;
                self.reserved(0, padding)?;
                Ok(())
            }
            StringMode::NullTerminated => {
                self.write(&bytes)?;
                self.reserved(0, 1)?;
                Ok(())
            }
        }
    }

    pub fn utf16_str<T: AsRef<str>>(&mut self, value: T, mode: StringMode) -> crate::writer::Result<()> {
        const CHAR_SIZE: usize = std::mem::size_of::<u16>();
        let bytes = E::u16iter_to_bytes(value.as_ref().encode_utf16(), value.as_ref().len() * CHAR_SIZE);
        match mode {
            StringMode::FixedChars(size) => {
                if bytes.len() > size * CHAR_SIZE {
                    return Err(crate::result::WriteError::io_error(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "String is too long for fixed size",
                    )));
                }
                let padding = size * CHAR_SIZE - bytes.len();
                self.write(&bytes)?;
                self.reserved(0, padding)?;
                Ok(())
            }
            StringMode::NullTerminated => {
                self.write(&bytes)?;
                self.reserved(0, CHAR_SIZE)?;
                Ok(())
            }
        }
    }
}
impl<E: Endian, W: crate::async_writer::WriterBase> AsyncBinaryWriter<E, W> {
    pub async fn utf8_str<T: AsRef<str>>(&mut self, value: T, mode: StringMode) -> crate::async_writer::Result<()> {
        let bytes = value.as_ref().as_bytes();
        match mode {
            StringMode::FixedChars(size) => {
                if bytes.len() > size {
                    return Err(crate::result::WriteError::io_error(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "String is too long for fixed size",
                    )));
                }
                let padding = size - bytes.len();
                self.write(&bytes).await?;
                self.reserved(0, padding).await?;
            }
            StringMode::NullTerminated => {
                self.write(&bytes).await?;
                self.reserved(0, 1).await?;
            }
        }
        Ok(())
    }

    pub async fn utf16_str<T: AsRef<str>>(&mut self, value: T, mode: StringMode) -> crate::async_writer::Result<()> {
        const CHAR_SIZE: usize = std::mem::size_of::<u16>();
        let bytes = E::u16iter_to_bytes(value.as_ref().encode_utf16(), value.as_ref().len() * CHAR_SIZE);
        match mode {
            StringMode::FixedChars(size) => {
                if bytes.len() > size * CHAR_SIZE {
                    return Err(crate::result::WriteError::io_error(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "String is too long for fixed size",
                    )));
                }
                let padding = size * CHAR_SIZE - bytes.len();
                self.write(&bytes).await?;
                self.reserved(0, padding).await?;
            }
            StringMode::NullTerminated => {
                self.write(&bytes).await?;
                self.reserved(0, CHAR_SIZE).await?;
            }
        }
        Ok(())
    }
}

#[tokio::test]
async fn test_async_write_strings() {
    use std::io::Cursor;
    let mut data = vec![0; 16];

    // UTF-8 Null-terminated string
    let stream = tokio::io::BufStream::new(Cursor::new(&mut data));
    let mut writer = AsyncBinaryWriter::new_be(stream);
    assert!(writer.utf8_str("Hello", StringMode::NullTerminated).await.is_ok());
    assert!(writer.utf8_str("World", StringMode::NullTerminated).await.is_ok());
    writer.flush().await.ok();
    // check the written data
    let expected = [
        72, 101, 108, 108, 111, 0, // "Hello\0"
        87, 111, 114, 108, 100, 0, // "World\0"
        0, 0, 0, 0,
    ];
    assert_eq!(&data[..], &expected[..]);

    // UTF-8 Fixed size string
    let stream = tokio::io::BufStream::new(Cursor::new(&mut data));
    let mut writer = AsyncBinaryWriter::new_be(stream);
    assert!(writer.utf8_str("Hello", StringMode::FixedChars(10)).await.is_ok());
    assert!(writer.utf8_str("World", StringMode::FixedChars(10)).await.is_ok());
    writer.flush().await.ok();
    // check the written data
    let expected = [
        72, 101, 108, 108, 111, 0, 0, 0, 0, 0, // "Hello\0\0\0\0"
        87, 111, 114, 108, 100, 0, 0, 0, 0, 0, // "World\0\0\0\0"
    ];
    assert_eq!(&data[..], &expected[..]);

    // UTF-16 Null-terminated string
    let stream = tokio::io::BufStream::new(Cursor::new(&mut data));
    let mut writer = AsyncBinaryWriter::new_be(stream);
    assert!(writer.utf16_str("Hello", StringMode::NullTerminated).await.is_ok());
    assert!(writer.utf16_str("World", StringMode::NullTerminated).await.is_ok());
    writer.flush().await.ok();
    // check the written data
    let expected = [
        0, b'H', 0, b'e', 0, b'l', 0, b'l', 0, b'o', 0, 0, // "Hello\0"
        0, b'W', 0, b'o', 0, b'r', 0, b'l', 0, b'd', 0, 0, // "World\0"
    ];
    assert_eq!(&data[..], &expected[..]);

    // UTF-16 Fixed size string
    let stream = tokio::io::BufStream::new(Cursor::new(&mut data));
    let mut writer = AsyncBinaryWriter::new_be(stream);
    assert!(writer.utf16_str("Hello", StringMode::FixedChars(6)).await.is_ok());
    assert!(writer.utf16_str("World", StringMode::FixedChars(6)).await.is_ok());
    writer.flush().await.ok();
    // check the written data
    let expected = [
        0, b'H', 0, b'e', 0, b'l', 0, b'l', 0, b'o', 0, 0, // "Hello\0\0"
        0, b'W', 0, b'o', 0, b'r', 0, b'l', 0, b'd', 0, 0, // "World\0\0"
    ];
    assert_eq!(&data[..], &expected[..]);
}

#[test]
fn test_read_strings() {
    let data = b"Hello, world!\0";
    assert_eq!(BinaryReader::from_le_bytes(data).utf8_str(StringMode::NullTerminated).unwrap(), "Hello, world!");
    assert_eq!(BinaryReader::from_le_bytes(data).utf8_str(StringMode::FixedChars(13)).unwrap(), "Hello, world!");

    let data = b"Hello, world!\0";
    assert_eq!(BinaryReader::from_be_bytes(data).utf8_str(StringMode::NullTerminated).unwrap(), "Hello, world!");
    assert_eq!(BinaryReader::from_be_bytes(data).utf8_str(StringMode::FixedChars(13)).unwrap(), "Hello, world!");

    let data = b"H\0e\0l\0l\0o\0,\0 \0w\0o\0r\0l\0d\0!\0\0\0";
    assert_eq!(BinaryReader::from_le_bytes(data).utf16_str(StringMode::NullTerminated).unwrap(), "Hello, world!");
    assert_eq!(BinaryReader::from_le_bytes(data).utf16_str(StringMode::FixedChars(13)).unwrap(), "Hello, world!");

    let data = b"\0H\0e\0l\0l\0o\0,\0 \0w\0o\0r\0l\0d\0!\0\0\0";
    assert_eq!(BinaryReader::from_be_bytes(data).utf16_str(StringMode::NullTerminated).unwrap(), "Hello, world!");
    assert_eq!(BinaryReader::from_be_bytes(data).utf16_str(StringMode::FixedChars(13)).unwrap(), "Hello, world!");
}

#[test]
fn test_write_strings() {
    let mut buf = Vec::new();
    let mut writer = BinaryWriter::new_le(&mut buf);

    writer.utf8_str("Hello", StringMode::NullTerminated).unwrap();
    writer.utf16_str("World", StringMode::NullTerminated).unwrap();

    assert_eq!(buf, b"Hello\0W\0o\0r\0l\0d\0\0\0");
}
