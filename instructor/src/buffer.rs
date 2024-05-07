use std::ops::DerefMut;
use bytes::{Buf, BufMut};
use crate::{Endian, Error, Instruct, Exstruct};

pub trait Buffer {
    fn try_copy_to_slice(&mut self, buf: &mut [u8]) -> Result<(), Error>;

    fn remaining(&self) -> usize;

    fn read<T, E>(&mut self) -> Result<T, Error>
        where
            T: Exstruct<E>,
            E: Endian,
    {
        T::read_from_buffer(self)
    }
}

//impl Buffer for &[u8] {
//    fn copy_to_slice(&mut self, buf: &mut [u8]) -> Result<(), Error> {
//        if self.len() < buf.len() {
//            return Err(Error::TooShort);
//        }
//        let (data, rest) = self.split_at(buf.len());
//        *self = rest;
//        buf.copy_from_slice(data);
//        Ok(())
//    }
//
//    fn remaining(&self) -> Option<usize> {
//        Some(self.len())
//    }
//}

impl<T: Buf> Buffer for T {
    fn try_copy_to_slice(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        if Buf::remaining(self) < buf.len() {
            return Err(Error::TooShort);
        }
        self.copy_to_slice(buf);
        Ok(())
    }

    fn remaining(&self) -> usize {
        Buf::remaining(self)
    }
}

pub trait BufferMut {
    fn extend_from_slice(&mut self, buf: &[u8]);

    fn write<T, E>(&mut self, value: &T)
        where
            T: Instruct<E>,
            E: Endian,
    {
        value.write_to_buffer(self);
    }
}

impl<T: BufMut> BufferMut for T {
    fn extend_from_slice(&mut self, buf: &[u8]) {
        self.put_slice(buf);
    }
}

pub trait DoubleEndedBufferMut: BufferMut {
    fn write_front<T, E>(&mut self, value: &T)
        where
            T: Instruct<E>,
            E: Endian;
}

impl<T: BufferMut + DerefMut<Target = [u8]>> DoubleEndedBufferMut for T {
    fn write_front<U, E>(&mut self, value: &U)
        where
            U: Instruct<E>,
            E: Endian,
    {
        let len = self.len();
        self.write(value);
        let diff = self.len() - len;
        self.rotate_right(diff);
    }
}