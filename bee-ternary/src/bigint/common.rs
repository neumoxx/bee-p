use crate::bigint::private;

/// The number of bits in an I384
pub const BINARY_LEN: usize = 384;

/// The number of bytes in an I384
pub const BINARY_LEN_IN_U8: usize = BINARY_LEN / 8;
pub const BINARY_LEN_IN_U32: usize = BINARY_LEN / 32;

/// The inner representation of a I384 using 48 u8s.
pub type U8Repr = [u8; BINARY_LEN_IN_U8];

/// The inner representation of a I384 using 12 u32s.
pub type U32Repr = [u32; BINARY_LEN_IN_U32];

#[derive(Clone, Copy, Debug)]
pub struct BigEndian {}

#[derive(Clone, Copy, Debug)]
pub struct LittleEndian {}

trait EndianType: private::Sealed {}

impl EndianType for BigEndian {}
impl EndianType for LittleEndian {}

pub trait BinaryRepresentation: private::Sealed + Clone {
    type T;
    fn iter(&self) -> std::slice::Iter<'_, Self::T>;
}

impl private::Sealed for U8Repr {}
impl private::Sealed for U32Repr {}

impl BinaryRepresentation for U8Repr {
    type T = u8;

    fn iter(&self) -> std::slice::Iter<'_, Self::T> {
        (self as &[u8]).iter()
    }
}

impl BinaryRepresentation for U32Repr {
    type T = u32;

    fn iter(&self) -> std::slice::Iter<'_, Self::T> {
        (self as &[u32]).iter()
    }
}

#[derive(Clone, Debug)]
pub enum Error {
    BinaryExceedsTernaryRange,
    TernaryExceedsBinaryRange,
}

