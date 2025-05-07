//! rwbin: Fast, zero-copy binary reader and writer for Rust.
//!
//! Provides both synchronous and asynchronous APIs for reading and writing binary data
//! in little-endian or big-endian formats. Supports primitive types, tuples, arrays,
//! vectors, strings (UTF-8/UTF-16), and custom Read/Write traits for complex data types.
//!
//! # Examples
//!
//! Sync example:
//! ```rust
//! use rwbin::reader::BinaryReader;
//! use rwbin::writer::BinaryWriter;
//! use rwbin::endian::LittleEndian;
//!
//! let mut buf = Vec::new();
//! let mut w = BinaryWriter::<LittleEndian, _>::new_le(&mut buf);
//! w.u32(0xDEADBEEF).unwrap();
//! w.i16(-123).unwrap();
//! w.flush().unwrap();
//!
//! let mut r = BinaryReader::<LittleEndian, _>::from_le_bytes(&buf);
//! assert_eq!(r.u32().unwrap(), 0xDEADBEEF);
//! assert_eq!(r.i16().unwrap(), -123);
//! ```
//!
//! Async example (ignore in doc-tests):
//! ```rust,ignore
//! use rwbin::async_reader::AsyncBinaryReader;
//! use rwbin::endian::BigEndian;
//! use tokio::io::BufReader;
//!
//! #[tokio::main]
//! async fn main() {
//!     let data = vec![0x00, 0x01, 0x00, 0x02];
//!     let mut r = AsyncBinaryReader::<BigEndian, _>::from_be_bytes(&data);
//!     let x: u16 = r.u16().await.unwrap();
//!     let y: u16 = r.u16().await.unwrap();
//!     assert_eq!(x, 1);
//!     assert_eq!(y, 2);
//! }
//! ```
/// Asynchronous binary reader supporting futures-based I/O.
///
/// Construct with `AsyncBinaryReader::<Endian, _>::new_le`, `new_be`, `from_le_bytes`, or
/// `from_be_bytes`, then call methods like `.u8()`, `.read::<T>()`, or implement `AsyncRead` for your types.
pub mod async_reader;
/// Asynchronous binary writer supporting futures-based I/O.
///
/// Construct with `AsyncBinaryWriter::<Endian, _>::new_le`, `new_be`, then call methods like
/// `.u8()`, `.write::<T>()`, or implement `AsyncWrite` for your types.
pub mod async_writer;
/// Endianness utilities for byte conversions.
///
/// Contains `LittleEndian` and `BigEndian` types implementing the `Endian` trait,
/// which converts primitives to/from byte arrays.
pub mod endian;
/// Synchronous binary reader wrapping any `BufRead`.
///
/// Construct with `BinaryReader::<Endian, _>::new_le`, `new_be`, `from_le_bytes`, or `from_be_bytes`,
/// then call methods like `.u8()`, `.read::<T>()`, `.skip()`, etc.
pub mod reader;
/// Definitions of result and error types for binary I/O.
///
/// Includes `ReadError`, `WriteError`, and the `ReadResult` / `WriteResult` aliases.
pub mod result;
/// String utilities for reading and writing UTF-8 and UTF-16 data.
///
/// Provides `utf8_str` and `utf16_str` methods on readers/writers for fixed-length or
/// null-terminated strings in sync and async contexts.
pub mod string;
/// Synchronous binary writer wrapping any `Write`.
///
/// Construct with `BinaryWriter::<Endian, _>::new_le`, `new_be`, then call methods like
/// `.u8()`, `.write::<T>()`, `.flush()`, or implement `Write` for custom types.
pub mod writer;

mod tests;
