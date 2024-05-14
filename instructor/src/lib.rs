mod bitfield;
mod buffer;
mod error;
mod pack;
mod unpack;
pub mod utils;

pub use bitfield::{BitBuffer, BitStorage};
pub use buffer::{Buffer, BufferMut, DoubleEndedBufferMut};
pub use error::Error;
#[cfg(feature = "derive")]
pub use instructor_derive::{Exstruct, Instruct};
pub use pack::Instruct;
pub use unpack::Exstruct;

pub struct LittleEndian;
pub struct BigEndian;

#[cfg(target_endian = "little")]
pub type NativeEndian = LittleEndian;

#[cfg(target_endian = "big")]
pub type NativeEndian = BigEndian;

pub type NetworkEndian = BigEndian;

pub trait Endian: unpack::ReadPrimitive + pack::WritePrimitive + map::MapIndex {}
impl Endian for LittleEndian {}
impl Endian for BigEndian {}

mod map {
    use std::ops::Range;

    use crate::{BigEndian, LittleEndian};

    pub trait MapIndex {
        fn map_index(n: usize, m: usize) -> Range<usize>;
    }

    impl MapIndex for LittleEndian {
        fn map_index(n: usize, _: usize) -> Range<usize> {
            0..n
        }
    }

    impl MapIndex for BigEndian {
        fn map_index(n: usize, m: usize) -> Range<usize> {
            (m - n)..m
        }
    }
}
