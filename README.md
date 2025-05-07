# rwbin

A lightweight, binary reader and writer for Rust — helping you handle sync/async I/O with ease and speed.

## Features

- Easy-to-use synchronous and asynchronous readers/writers
- Little- and big-endian operations
- Read/write of primitive types, tuples, arrays, vectors, and options
- String read/write: UTF-8 / UTF-16, fixed-length or null-terminated
- Extendable via `Read`/`Write` and `AsyncRead`/`AsyncWrite` traits

## 🚀 Quick Start

### Installation
Add this to your `Cargo.toml`:

```toml
[dependencies]
rwbin = "0.1"
```

### Synchronous example

```rust
use rwbin::reader::BinaryReader;
use rwbin::writer::BinaryWriter;
use rwbin::endian::LittleEndian;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = Vec::new();
    let mut writer = BinaryWriter::<LittleEndian, _>::new_le(&mut buf);
    writer.u32(0xDEADBEEF)?;
    writer.i8(-42)?;
    writer.flush()?;

    let mut reader = BinaryReader::<LittleEndian, _>::from_le_bytes(&buf);
    let a = reader.u32()?;
    let b = reader.i8()?;

    assert_eq!(a, 0xDEADBEEF);
    assert_eq!(b, -42);
    Ok(())
}
```

### Asynchronous example

```rust
use rwbin::async_reader::AsyncBinaryReader;
use rwbin::endian::BigEndian;
use tokio::io::BufReader;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = vec![0x00, 0x01, 0x00, 0x02];
    let mut reader = AsyncBinaryReader::<BigEndian, _>::from_be_bytes(&data);
    let x: u16 = reader.u16().await?;
    let y: u16 = reader.u16().await?;
    assert_eq!(x, 1);
    assert_eq!(y, 2);
    Ok(())
}
```

## Crate Modules

- `reader` / `async_reader`: `BinaryReader` / `AsyncBinaryReader`
- `writer` / `async_writer`: `BinaryWriter` / `AsyncBinaryWriter`
- `endian`: `LittleEndian` / `BigEndian`
- `string`: UTF-8 / UTF-16 string utilities
- `result`: `ReadError`, `WriteError` and result aliases

## License

MIT. See [LICENSE](LICENSE) for details.
