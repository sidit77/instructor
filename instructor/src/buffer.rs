use crate::{Endian, Error, Unpack};

pub trait Buffer {
    fn get(&mut self, size: usize) -> Result<&[u8], Error>;

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
    fn get(&mut self, size: usize) -> Result<&[u8], Error> {
        if self.len() < size {
            return Err(Error::TooShort);
        }
        let (data, rest) = self.split_at(size);
        *self = rest;
        Ok(data)
    }

    fn remaining(&self) -> Option<usize> {
        Some(self.len())
    }
}