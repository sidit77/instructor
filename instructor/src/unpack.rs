use crate::{Buffer, Endian, NativeEndian, Error};

pub trait Unpack<E: Endian>
    where
        Self: Sized,
{
    fn unpack<B: Buffer + ?Sized>(buffer: &mut B) -> Result<Self, Error>;
}

impl<E: Endian, const N: usize> Unpack<E> for [u8; N] {
    fn unpack<B: Buffer + ?Sized>(buffer: &mut B) -> Result<Self, Error> {
        let mut array = [0; N];
        buffer.copy_to_slice(&mut array)?;
        Ok(array)
    }
}

macro_rules! impl_int_unpack {
    ($($t:ident),+) => {
        $(
            impl<E: Endian> Unpack<E> for $t {
                fn unpack<B: Buffer + ?Sized>(buffer: &mut B) -> Result<Self, Error> {
                    Ok(E::$t(Unpack::<NativeEndian>::unpack(buffer)?))
                }
            }
        )*
    }
}

impl_int_unpack!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
