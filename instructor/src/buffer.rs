use crate::{Endian, Error, Unpack};

pub trait Buffer {
    fn copy_to_slice(&mut self, buf: &mut [u8]) -> Result<(), Error>;

    fn remaining(&self) -> Option<usize>;

    fn read<T, E>(&mut self) -> Result<T, Error>
        where
            T: Unpack<E>,
            E: Endian,
    {
        T::unpack(self)
    }
}

impl Buffer for &[u8] {
    fn copy_to_slice(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        if self.len() < buf.len() {
            return Err(Error::TooShort);
        }
        let (data, rest) = self.split_at(buf.len());
        *self = rest;
        buf.copy_from_slice(data);
        Ok(())
    }

    fn remaining(&self) -> Option<usize> {
        Some(self.len())
    }
}