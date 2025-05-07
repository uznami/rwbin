// Synchronous Example
#[test]
fn test_sync_read_write() -> Result<(), Box<dyn std::error::Error>> {
    use crate::endian::Endian;
    use crate::reader::{self, BinaryReader};
    use crate::string::StringMode;
    use crate::writer::{self, BinaryWriter};
    use std::fs::File;
    use std::io::{BufRead, BufReader, BufWriter, Write};
    use std::path::PathBuf;

    struct TestStruct {
        points: [(f32, f32); 4],
        items: Vec<Vec<String>>,
    }
    impl reader::Read for TestStruct {
        fn read<E: Endian, R: BufRead>(reader: &mut BinaryReader<E, R>) -> reader::Result<Self> {
            let points = reader.read()?;
            let count = reader.u32()? as usize;
            let mut items = Vec::with_capacity(count);
            let num_items_array: Vec<u32> = reader.read_with(count)?;
            for num_items in num_items_array.iter() {
                let mut item_vec = Vec::with_capacity(*num_items as usize);
                let num_strs = reader.u32()? as usize;
                let str_len_array: Vec<u32> = reader.read_with(num_strs)?;
                for str_len in str_len_array.iter() {
                    let s = reader.utf8_str(StringMode::FixedChars(*str_len as usize))?;
                    item_vec.push(s);
                }
                items.push(item_vec);
            }
            Ok(TestStruct { points, items })
        }
    }
    impl writer::Write for TestStruct {
        fn write<E: Endian, W: Write>(&self, writer: &mut BinaryWriter<E, W>) -> writer::Result<()> {
            writer.write(&self.points)?;
            let count = self.items.len() as u32;
            writer.u32(count)?;
            let num_items_array = self.items.iter().map(|item_vec| item_vec.len() as u32).collect::<Vec<u32>>();
            writer.write(&num_items_array)?;
            for item_vec in &self.items {
                let num_items = item_vec.len() as u32;
                writer.u32(num_items)?;
                let str_len_array = item_vec.iter().map(|s| s.len() as u32).collect::<Vec<u32>>();
                writer.write(&str_len_array)?;
                for s in item_vec.iter() {
                    writer.utf8_str(s, StringMode::FixedChars(s.len()))?;
                }
                writer.write(&str_len_array)?;
            }
            writer.write(&num_items_array)?;
            writer.write(&self.points)?;
            Ok(())
        }
    }

    let path = PathBuf::from("test.bin");
    let file = File::create(&path)?;
    let buf_writer = BufWriter::new(file);
    let mut writer = BinaryWriter::new_le(buf_writer);
    writer.u32(0xDEADBEEF)?;
    writer.i16(-42)?;
    writer.flush()?;

    let file = File::open(&path)?;
    let buf_reader = BufReader::new(file);
    let mut reader = BinaryReader::new_le(buf_reader);
    assert_eq!(reader.u32()?, 0xDEADBEEF);
    assert_eq!(reader.i16()?, -42);

    std::fs::remove_file(&path)?;

    Ok(())
}

// Async Example
#[tokio::test]
async fn test_async_read_write() -> Result<(), Box<dyn std::error::Error>> {
    use crate::async_reader::AsyncBinaryReader;
    use crate::async_writer::AsyncBinaryWriter;
    use std::path::PathBuf;
    use tokio::fs::File;
    use tokio::io::{BufReader, BufWriter};

    let path = PathBuf::from("test_async.bin");
    let file = File::create(&path).await?;
    let buf_writer = BufWriter::new(file);
    let mut writer = AsyncBinaryWriter::new_le(buf_writer);
    writer.u32(0xDEADBEEF).await?;
    writer.i16(-42).await?;
    writer.flush().await?;

    let file = File::open(&path).await?;
    let buf_reader = BufReader::new(file);
    let mut reader = AsyncBinaryReader::new_le(buf_reader);
    assert_eq!(reader.u32().await?, 0xDEADBEEF);
    assert_eq!(reader.i16().await?, -42);

    tokio::fs::remove_file(&path).await?;

    Ok(())
}
