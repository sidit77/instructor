use std::fmt::{Debug, Display, Formatter, LowerHex, UpperHex};
use std::ops::Deref;

use crate::pack::WritePrimitive;
use crate::unpack::ReadPrimitive;
use crate::{BitStorage, Buffer, BufferMut, Endian, Error, Exstruct, Instruct};

#[derive(Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Length<T, const OFFSET: isize>(T);

impl<T, const OFFSET: isize> Length<T, OFFSET>
where
    T: TryFrom<usize>
{
    pub fn new(len: usize) -> Result<Self, Error> {
        let len = T::try_from(len).map_err(|_| Error::InvalidValue)?;
        Ok(Self(len))
    }

    pub fn with_offset(len: usize) -> Result<Self, Error> {
        Self::new(len.saturating_add_signed(-OFFSET))
    }
}

impl<E: Endian, T, const OFFSET: isize> Exstruct<E> for Length<T, OFFSET>
where
    T: TryInto<usize> + Exstruct<E> + Copy
{
    #[inline]
    fn read_from_buffer<B: Buffer + ?Sized>(buffer: &mut B) -> Result<Self, Error> {
        let len = buffer.read::<T, E>()?;
        buffer
            .remaining()
            .eq(&len
                .try_into()
                .map_err(|_| Error::InvalidValue)?
                .saturating_add_signed(OFFSET))
            .then_some(Self(len))
            .ok_or(Error::UnexpectedLength)
    }
}

impl<E: Endian, T, const OFFSET: isize> Instruct<E> for Length<T, OFFSET>
where
    T: Instruct<E>
{
    #[inline]
    fn write_to_buffer<B: BufferMut + ?Sized>(&self, buffer: &mut B) {
        buffer.write_ref(&self.0);
    }
}

impl<T: Debug, const OFFSET: isize> Debug for Length<T, OFFSET> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<T: Display, const OFFSET: isize> Display for Length<T, OFFSET> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<T, const OFFSET: isize> Deref for Length<T, OFFSET> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait DynBuffer {
    fn try_copy_to_slice(&mut self, buf: &mut [u8]) -> Result<(), Error>;

    fn remaining(&self) -> usize;
}

impl<T: Buffer> DynBuffer for T {
    fn try_copy_to_slice(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        T::try_copy_to_slice(self, buf)
    }

    fn remaining(&self) -> usize {
        T::remaining(self)
    }
}

pub struct Limit<'a> {
    buffer: &'a mut dyn DynBuffer,
    remaining: usize
}

impl<'a> Limit<'a> {
    pub fn new<B: DynBuffer>(buffer: &'a mut B, remaining: usize) -> Self {
        Self { buffer, remaining }
    }
}

impl<'a> Buffer for Limit<'a> {
    fn try_copy_to_slice(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        if self.remaining < buf.len() {
            return Err(Error::TooShort);
        }
        self.buffer.try_copy_to_slice(buf)?;
        self.remaining -= buf.len();
        Ok(())
    }

    fn skip(&mut self, n: usize) -> Result<(), Error> {
        if self.remaining < n {
            return Err(Error::TooShort);
        }
        self.remaining -= n;
        Ok(())
    }

    fn remaining(&self) -> usize {
        self.remaining.min(self.buffer.remaining())
    }
}

#[allow(non_camel_case_types)]
#[derive(Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct u24(u32);

impl u24 {
    pub const MAX: Self = Self(0x00FF_FFFF);
    pub const MIN: Self = Self(0x0000_0000);
    pub const BITS: u32 = 24;

    pub const fn new(value: u32) -> Self {
        assert!(value <= Self::MAX.0, "Value out of range");
        Self(value)
    }
}

impl<E: Endian> Exstruct<E> for u24 {
    #[inline]
    fn read_from_buffer<B: Buffer + ?Sized>(buffer: &mut B) -> Result<Self, Error> {
        let mut data = [0; 4];
        buffer.try_copy_to_slice(&mut data[E::map_index(3, 4)])?;
        Ok(Self(<E as ReadPrimitive>::u32(data)))
    }
}

impl<E: Endian> Instruct<E> for u24 {
    #[inline]
    fn write_to_buffer<B: BufferMut + ?Sized>(&self, buffer: &mut B) {
        let data = <E as WritePrimitive>::u32(self.0);
        buffer.extend_from_slice(&data[E::map_index(3, 4)]);
    }
}

impl Display for u24 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Debug for u24 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl LowerHex for u24 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        LowerHex::fmt(&self.0, f)
    }
}

impl UpperHex for u24 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        UpperHex::fmt(&self.0, f)
    }
}

impl From<u8> for u24 {
    fn from(value: u8) -> Self {
        Self(value as u32)
    }
}

impl From<u16> for u24 {
    fn from(value: u16) -> Self {
        Self(value as u32)
    }
}

impl TryFrom<u32> for u24 {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value > (Self::MAX.0) { Err(()) } else { Ok(Self(value)) }
    }
}

impl From<u24> for u32 {
    fn from(value: u24) -> Self {
        value.0
    }
}

impl BitStorage for u24 {
    type Buffer = [u8; 3];

    #[inline]
    fn extract(&self, start: u32, end: u32) -> Self::Buffer {
        debug_assert!(start < end);
        debug_assert!(end <= Self::BITS);
        let mask = (1 << (end - start)) - 1;
        let masked = (self.0 >> start) & mask;
        let bytes = masked.to_be_bytes();
        [bytes[1], bytes[2], bytes[3]]
    }

    fn insert(&mut self, start: u32, end: u32, value: Self::Buffer) {
        debug_assert!(start < end);
        debug_assert!(end <= Self::BITS);
        let mask = (1 << (end - start)) - 1;
        let value = u32::from_be_bytes([0, value[0], value[1], value[2]]);
        let masked = (value & mask) << start;
        *self = Self(self.0 | masked);
    }
}

