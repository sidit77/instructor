use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use crate::{Buffer, Endian, Error, Unpack};

#[derive(Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Length<T, const OFFSET: isize>(T);

impl<E: Endian, T, const OFFSET: isize> Unpack<E> for Length<T, OFFSET>
    where T: TryInto<usize> + Unpack<E> + Copy
{
    fn unpack<B: Buffer + ?Sized>(buffer: &mut B) -> Result<Self, Error> {
        let len = buffer.read::<T, E>()?;
        buffer
            .remaining()
            .expect("The length type can not be used with buffer with unknown size")
            .eq(&len
                .try_into()
                .map_err(|_| Error::InvalidValue)?
                .saturating_add_signed(OFFSET))
            .then_some(Self(len))
            .ok_or(Error::TooShort)
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