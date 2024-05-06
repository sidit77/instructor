
mod unpack;
mod error;
mod buffer;
pub mod utils;
mod bitfield;
mod pack;

pub use unpack::Unpack;
pub use pack::Pack;
pub use error::Error;
pub use buffer::{Buffer, BufferMut};
pub use bitfield::{BitBuffer, BitStorage};
#[cfg(feature = "derive")]
pub use instructor_derive::Unpack;


pub struct LittleEndian;
pub struct BigEndian;

#[cfg(target_endian = "little")]
pub type NativeEndian = LittleEndian;

#[cfg(target_endian = "big")]
pub type NativeEndian = BigEndian;

pub type NetworkEndian = BigEndian;

pub trait Endian: unpack::ReadPrimitive + pack::WritePrimitive {}
impl Endian for LittleEndian {}
impl Endian for BigEndian {}