use std::mem::size_of;
use crate::{Buffer, BufferMut, Endian, Error, Instruct, Exstruct};

pub trait BitStorage: Sized + Copy + Default {
    type Buffer: AsMut<[u8]> + Default;
    fn extract(&self, start: u32, end: u32) -> Self::Buffer;

    fn insert(&mut self, start: u32, end: u32, value: Self::Buffer);
}

macro_rules! impl_bitstorage_trait {
    ($($ty:ident),+) => {
        $(
            impl BitStorage for $ty {
                type Buffer = [u8; size_of::<Self>()];

                #[inline]
                fn extract(&self, start: u32, end: u32) -> Self::Buffer {
                    debug_assert!(start < end);
                    debug_assert!(end <= Self::BITS);
                    let mask = (1 << (end - start)) - 1;
                    let masked = (self >> start) & mask;
                    masked.to_be_bytes()
                }

                fn insert(&mut self, start: u32, end: u32, value: Self::Buffer) {
                    debug_assert!(start < end);
                    debug_assert!(end <= Self::BITS);
                    let mask = (1 << (end - start)) - 1;
                    let masked = (Self::from_be_bytes(value) & mask) << start;
                    *self |= masked;
                }

            }
        )*
    }
}

impl_bitstorage_trait!(u8, u16, u32, u64, u128);

pub struct BitBuffer<I: BitStorage> {
    storage: I,
    start: u32,
    end: u32,
    remaining: usize
}

impl<I: BitStorage> BitBuffer<I> {

    #[inline]
    pub fn new<E, B: Buffer + ?Sized>(source: &mut B) -> Result<Self, Error>
        where I: Exstruct<E>, E: Endian
    {
        Ok(Self {
            storage: Exstruct::<E>::read_from_buffer(source)?,
            start: 0,
            end: 0,
            remaining: source.remaining(),
        })
    }
}

impl<I: BitStorage> BitBuffer<I> {

    #[inline]
    pub fn empty() -> Self {
        Self {
            storage: I::default(),
            start: 0,
            end: 0,
            remaining: 0
        }
    }

    #[inline]
    pub fn set_range(&mut self, start: u32, end: u32) {
        self.start = start;
        self.end = end;
    }
}

impl<I: BitStorage> Buffer for BitBuffer<I> {

    #[inline]
    fn try_copy_to_slice(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        let mut shifted = self.storage.extract(self.start, self.end);
        let own = shifted.as_mut();
        if own.len() < buf.len() {
            return Err(Error::TooShort);
        }
        let start = own.len() - buf.len();
        buf.copy_from_slice(&own[start..]);
        Ok(())
    }

    #[inline]
    fn remaining(&self) -> usize {
        self.remaining
    }
}

impl<I: BitStorage> BufferMut for BitBuffer<I> {

    #[inline]
    fn extend_from_slice(&mut self, buf: &[u8]) {
        let mut buffer = I::Buffer::default();
        {
            let buffer = buffer.as_mut();
            let start = buffer
                .len()
                .checked_sub(buf.len())
                .expect("Datatype exceeds size of bitfield");
            buffer[start..].copy_from_slice(buf);
        }
        self.storage.insert(self.start, self.end, buffer);
    }
}

impl<I: BitStorage + Instruct<E>, E: Endian> Instruct<E> for BitBuffer<I> {
    fn write_to_buffer<B: BufferMut + ?Sized>(&self, buffer: &mut B) {
        Instruct::<E>::write_to_buffer(&self.storage, buffer);
    }
}