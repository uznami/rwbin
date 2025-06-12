use std::mem::size_of;

pub trait Endian {
    fn u16_to_bytes(value: u16) -> [u8; size_of::<u16>()];
    fn i16_to_bytes(value: i16) -> [u8; size_of::<i16>()];
    fn u32_to_bytes(value: u32) -> [u8; size_of::<u32>()];
    fn i32_to_bytes(value: i32) -> [u8; size_of::<i32>()];
    fn f32_to_bytes(value: f32) -> [u8; size_of::<f32>()];
    fn u64_to_bytes(value: u64) -> [u8; size_of::<u64>()];
    fn i64_to_bytes(value: i64) -> [u8; size_of::<i64>()];
    fn f64_to_bytes(value: f64) -> [u8; size_of::<f64>()];

    fn u16iter_to_bytes<T: Iterator<Item = u16>>(iter: T, capacity: usize) -> Vec<u8> {
        let mut result = Vec::with_capacity(capacity);
        for e in iter {
            result.extend_from_slice(&Self::u16_to_bytes(e));
        }
        result
    }

    fn u16_from_bytes(bytes: &[u8; size_of::<u16>()]) -> u16;
    fn i16_from_bytes(bytes: &[u8; size_of::<i16>()]) -> i16;
    fn u32_from_bytes(bytes: &[u8; size_of::<u32>()]) -> u32;
    fn i32_from_bytes(bytes: &[u8; size_of::<i32>()]) -> i32;
    fn f32_from_bytes(bytes: &[u8; size_of::<f32>()]) -> f32;
    fn u64_from_bytes(bytes: &[u8; size_of::<u64>()]) -> u64;
    fn i64_from_bytes(bytes: &[u8; size_of::<i64>()]) -> i64;
    fn f64_from_bytes(bytes: &[u8; size_of::<f64>()]) -> f64;

    fn u16vec_from_bytes(bytes: &[u8]) -> Vec<u16> {
        assert!(bytes.len() % 2 == 0, "Invalid length for u16 array: {}", bytes.len());
        let mut result = Vec::with_capacity(bytes.len() / 2);
        for chunk in bytes.chunks_exact(2) {
            let value = Self::u16_from_bytes(chunk.try_into().expect("slice with incorrect length"));
            result.push(value);
        }
        result
    }
}

macro_rules! impl_to_bytes {
    ($($fn:ident: ($t:ty, $conv:ident)),* $(,)?) => {
        $(
            #[inline]
            fn $fn(value: $t) -> [u8; size_of::<$t>()] {
                value.$conv()
            }
        )*
    };
}

macro_rules! impl_from_bytes {
    ($($fn:ident: ($t:ty, $conv:ident)),* $(,)?) => {
        $(
            #[inline]
            fn $fn(bytes: &[u8; size_of::<$t>()]) -> $t {
                <$t>::$conv(*bytes)
            }
        )*
    };
}
pub struct LittleEndian;
impl Endian for LittleEndian {
    impl_to_bytes! {
        u16_to_bytes: (u16, to_le_bytes),
        i16_to_bytes: (i16, to_le_bytes),
        u32_to_bytes: (u32, to_le_bytes),
        i32_to_bytes: (i32, to_le_bytes),
        f32_to_bytes: (f32, to_le_bytes),
        u64_to_bytes: (u64, to_le_bytes),
        i64_to_bytes: (i64, to_le_bytes),
        f64_to_bytes: (f64, to_le_bytes),
    }

    impl_from_bytes! {
        u16_from_bytes: (u16, from_le_bytes),
        i16_from_bytes: (i16, from_le_bytes),
        u32_from_bytes: (u32, from_le_bytes),
        i32_from_bytes: (i32, from_le_bytes),
        f32_from_bytes: (f32, from_le_bytes),
        u64_from_bytes: (u64, from_le_bytes),
        i64_from_bytes: (i64, from_le_bytes),
        f64_from_bytes: (f64, from_le_bytes),
    }
}

pub struct BigEndian;
impl Endian for BigEndian {
    impl_to_bytes! {
        u16_to_bytes: (u16, to_be_bytes),
        i16_to_bytes: (i16, to_be_bytes),
        u32_to_bytes: (u32, to_be_bytes),
        i32_to_bytes: (i32, to_be_bytes),
        f32_to_bytes: (f32, to_be_bytes),
        u64_to_bytes: (u64, to_be_bytes),
        i64_to_bytes: (i64, to_be_bytes),
        f64_to_bytes: (f64, to_be_bytes),
    }

    impl_from_bytes! {
        u16_from_bytes: (u16, from_be_bytes),
        i16_from_bytes: (i16, from_be_bytes),
        u32_from_bytes: (u32, from_be_bytes),
        i32_from_bytes: (i32, from_be_bytes),
        f32_from_bytes: (f32, from_be_bytes),
        u64_from_bytes: (u64, from_be_bytes),
        i64_from_bytes: (i64, from_be_bytes),
        f64_from_bytes: (f64, from_be_bytes),
    }
}
