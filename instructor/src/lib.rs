pub trait Buffer {
    fn get(&mut self, size: usize) -> Result<&[u8], Error>;

    fn read<T, E>(&mut self) -> Result<T, Error>
        where
            T: Deserialize<E>,
            E: Endian,
    {
        T::deserialize(self)
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
}

pub trait Deserialize<E: Endian>
    where
        Self: Sized,
{
    fn deserialize<B: Buffer + ?Sized>(buffer: &mut B) -> Result<Self, Error>;
}

impl<E: Endian, const N: usize> Deserialize<E> for [u8; N] {
    fn deserialize<B: Buffer + ?Sized>(buffer: &mut B) -> Result<Self, Error> {
        let mut array = [0; N];
        array.copy_from_slice(buffer.get(N)?);
        Ok(array)
    }
}


pub trait Integer {
    const SIZE: usize;
    fn from_ne_slice(bytes: &[u8]) -> Self;
    fn swap_bytes(self) -> Self;
}

macro_rules! impl_deserialize {
    ($($t:ty),+) => {
        $(
            impl Integer for $t {
                const SIZE: usize = std::mem::size_of::<Self>();

                #[inline]
                fn from_ne_slice(bytes: &[u8]) -> Self {
                    let mut buf = [0; Self::SIZE];
                    buf.copy_from_slice(bytes);
                    Self::from_ne_bytes(buf)
                }

                #[inline]
                fn swap_bytes(self) -> Self {
                    self.swap_bytes()
                }
            }
        )*
    }
}

impl_deserialize!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

impl<E: Endian, I: Integer> Deserialize<E> for I {
    fn deserialize<B: Buffer + ?Sized>(buffer: &mut B) -> Result<Self, Error> {
        Ok(E::from_ne(I::from_ne_slice(buffer.get(I::SIZE)?)))
    }
}

pub trait Endian {
    fn from_ne<I: Integer>(i: I) -> I;
}

pub enum LittleEndian {}

impl Endian for LittleEndian {
    #[inline]
    fn from_ne<I: Integer>(i: I) -> I {
        #[cfg(target_endian = "little")]
        {
            i
        }
        #[cfg(not(target_endian = "little"))]
        {
            i.swap_bytes()
        }
    }
}

pub enum BigEndian {}

impl Endian for BigEndian {
    #[inline]
    fn from_ne<I: Integer>(i: I) -> I {
        #[cfg(target_endian = "big")]
        {
            i
        }
        #[cfg(not(target_endian = "big"))]
        {
            i.swap_bytes()
        }
    }
}

#[cfg(target_endian = "little")]
pub type NativeEndian = LittleEndian;

#[cfg(target_endian = "big")]
pub type NativeEndian = BigEndian;

#[derive(Debug)]
pub enum Error {
    TooShort,
}
