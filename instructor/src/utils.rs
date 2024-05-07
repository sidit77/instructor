use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use crate::{Buffer, BufferMut, Endian, Error, Instruct, Exstruct};

#[derive(Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Length<T, const OFFSET: isize>(T);

impl<T, const OFFSET: isize> Length<T, OFFSET>
    where T: TryFrom<usize>
{
    pub fn with_offset(len: usize) -> Result<Self, Error> {
        let len = T::try_from(len.saturating_add_signed(-OFFSET)).map_err(|_| Error::InvalidValue)?;
        Ok(Self(len))
    }
}

impl<E: Endian, T, const OFFSET: isize> Exstruct<E> for Length<T, OFFSET>
    where T: TryInto<usize> + Exstruct<E> + Copy
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
    where T: Instruct<E>
{
    #[inline]
    fn write_to_buffer<B: BufferMut + ?Sized>(&self, buffer: &mut B) {
        buffer.write(&self.0);
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