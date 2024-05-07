
mod unpack;
mod error;
mod buffer;
pub mod utils;
mod bitfield;
mod pack;

pub use unpack::Exstruct;
pub use pack::Instruct;
pub use error::Error;
pub use buffer::{Buffer, BufferMut, DoubleEndedBufferMut};
pub use bitfield::{BitBuffer, BitStorage};
#[cfg(feature = "derive")]
pub use instructor_derive::{Exstruct, Instruct};


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