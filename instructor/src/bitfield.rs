use std::mem::size_of;
use crate::{Buffer, Endian, Error, Unpack};

pub trait BitStorage: Sized + Copy {
    type BUFFER: AsRef<[u8]> + Default;
    fn mask(&self, start: usize, end: usize) -> Self::BUFFER;
}

macro_rules! impl_bitstorage_trait {
    ($($ty:ident),+) => {
        $(
            impl BitStorage for $ty {
                type BUFFER = [u8; size_of::<Self>()];

                fn mask(&self, start: usize, end: usize) -> Self::BUFFER {
                    debug_assert!(start < end);
                    debug_assert!(end <= Self::BITS as usize);
                    let mask = (1 << (end - start)) - 1;
                    let masked = (self >> start) & mask;
                    masked.to_be_bytes()
                }
            }
        )*
    }
}

impl_bitstorage_trait!(u8, u16, u32, u64, u128);

pub struct BitBuffer<I: BitStorage> {
    storage: I,
    buffer: I::BUFFER,
    remaining: Option<usize>
}

impl<I: BitStorage> BitBuffer<I> {
    pub fn new<E, B: Buffer + ?Sized>(source: &mut B) -> Result<Self, Error>
        where I: Unpack<E>, E: Endian
    {
        Ok(Self {
            storage: Unpack::<E>::unpack(source)?,
            buffer: I::BUFFER::default(),
            remaining: source.remaining() })
    }
}

impl<I: BitStorage> BitBuffer<I> {
    pub fn set_range(&mut self, start: usize, end: usize) {
        self.buffer = self.storage.mask(start, end);
    }
}

impl<I: BitStorage> Buffer for BitBuffer<I> {
    fn get(&mut self, size: usize) -> Result<&[u8], Error> {
        let start = self.buffer.as_ref().len() - size;
        self.buffer
            .as_ref()
            .get(start..)
            .ok_or(Error::TooShort)
    }

    fn remaining(&self) -> Option<usize> {
        self.remaining
    }
}
