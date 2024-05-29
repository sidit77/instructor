use std::mem::size_of;
use crate::{Buffer, Endian, Error, NativeEndian};

pub trait Exstruct<E: Endian>
where
    Self: Sized
{
    fn read_from_buffer<B: Buffer>(buffer: &mut B) -> Result<Self, Error>;
}

impl<E: Endian, const N: usize> Exstruct<E> for [u8; N] {
    #[inline]
    fn read_from_buffer<B: Buffer>(buffer: &mut B) -> Result<Self, Error> {
        let mut array = [0; N];
        buffer.try_copy_to_slice(&mut array)?;
        Ok(array)
    }
}

impl<E: Endian> Exstruct<E> for () {
    #[inline]
    fn read_from_buffer<B: Buffer>(_: &mut B) -> Result<Self, Error> {
        Ok(())
    }
}

impl<E: Endian> Exstruct<E> for bool {
    #[inline]
    fn read_from_buffer<B: Buffer>(buffer: &mut B) -> Result<Self, Error> {
        Ok(buffer.read::<u8, E>()? != 0)
    }
}

impl<E: Endian, T: Exstruct<E>> Exstruct<E> for Vec<T> {
    #[inline]
    fn read_from_buffer<B: Buffer>(buffer: &mut B) -> Result<Self, Error> {
        let mut vec = Vec::with_capacity(buffer.remaining() / size_of::<T>());
        while buffer.remaining() > 0 {
            vec.push(buffer.read::<T, E>()?);
        }
        Ok(vec)
    }

}

impl<E, T1, T2> Exstruct<E> for (T1, T2)
where
    E: Endian,
    T1: Exstruct<E>,
    T2: Exstruct<E>,
{
    #[inline]
    fn read_from_buffer<B: Buffer>(buffer: &mut B) -> Result<Self, Error> {
        Ok((T1::read_from_buffer(buffer)?, T2::read_from_buffer(buffer)?))
    }
}

impl<E, T1, T2, T3> Exstruct<E> for (T1, T2, T3)
where
    E: Endian,
    T1: Exstruct<E>,
    T2: Exstruct<E>,
    T3: Exstruct<E>,
{
    #[inline]
    fn read_from_buffer<B: Buffer>(buffer: &mut B) -> Result<Self, Error> {
        Ok((
            T1::read_from_buffer(buffer)?,
            T2::read_from_buffer(buffer)?,
            T3::read_from_buffer(buffer)?,
        ))
    }
}

impl<E, T1, T2, T3, T4> Exstruct<E> for (T1, T2, T3, T4)
where
    E: Endian,
    T1: Exstruct<E>,
    T2: Exstruct<E>,
    T3: Exstruct<E>,
    T4: Exstruct<E>,
{
    #[inline]
    fn read_from_buffer<B: Buffer>(buffer: &mut B) -> Result<Self, Error> {
        Ok((
            T1::read_from_buffer(buffer)?,
            T2::read_from_buffer(buffer)?,
            T3::read_from_buffer(buffer)?,
            T4::read_from_buffer(buffer)?,
        ))
    }
}

macro_rules! impl_prim_unpack {
    ($($t:ident),+) => {
        $(
            impl<E: Endian> Exstruct<E> for $t {
                #[inline]
                fn read_from_buffer<B: Buffer>(buffer: &mut B) -> Result<Self, Error> {
                    Ok(<E as ReadPrimitive>::$t(Exstruct::<NativeEndian>::read_from_buffer(buffer)?))
                }
            }
        )*
    }
}

impl_prim_unpack!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64);

macro_rules! gen_endian_trait {
    ($($ty:ident),+) => {
        pub trait ReadPrimitive {
            $(
                fn $ty(bytes: [u8; core::mem::size_of::<$ty>()]) -> $ty;
            )*
        }

        impl ReadPrimitive for crate::LittleEndian {
            $(
                #[inline(always)]
                fn $ty(bytes: [u8; core::mem::size_of::<$ty>()]) -> $ty {
                    $ty::from_le_bytes(bytes)
                }
            )*
        }

        impl ReadPrimitive for crate::BigEndian {
            $(
                #[inline(always)]
                fn $ty(bytes: [u8; core::mem::size_of::<$ty>()]) -> $ty {
                    $ty::from_be_bytes(bytes)
                }
            )*
        }

    }
}

gen_endian_trait!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64);
