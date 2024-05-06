use crate::{Buffer, Endian, NativeEndian, Error};

pub trait Unpack<E: Endian>
    where
        Self: Sized,
{
    fn unpack<B: Buffer + ?Sized>(buffer: &mut B) -> Result<Self, Error>;
}

impl<E: Endian, const N: usize> Unpack<E> for [u8; N] {
    #[inline]
    fn unpack<B: Buffer + ?Sized>(buffer: &mut B) -> Result<Self, Error> {
        let mut array = [0; N];
        buffer.try_copy_to_slice(&mut array)?;
        Ok(array)
    }
}

macro_rules! impl_prim_unpack {
    ($($t:ident),+) => {
        $(
            impl<E: Endian> Unpack<E> for $t {
                #[inline]
                fn unpack<B: Buffer + ?Sized>(buffer: &mut B) -> Result<Self, Error> {
                    Ok(<E as ReadPrimitive>::$t(Unpack::<NativeEndian>::unpack(buffer)?))
                }
            }
        )*
    }
}

impl_prim_unpack!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

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

gen_endian_trait!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);