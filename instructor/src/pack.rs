use bytes::Bytes;
use crate::{BufferMut, Endian};

pub trait Instruct<E: Endian>
where
    Self: Sized
{
    fn write_to_buffer<B: BufferMut>(&self, buffer: &mut B);
}

impl<E: Endian, const N: usize> Instruct<E> for [u8; N] {
    #[inline]
    fn write_to_buffer<B: BufferMut>(&self, buffer: &mut B) {
        buffer.extend_from_slice(self);
    }
}

impl<E: Endian> Instruct<E> for () {
    #[inline]
    fn write_to_buffer<B: BufferMut + ?Sized>(&self, _: &mut B) {}
}

impl<E: Endian> Instruct<E> for bool {
    #[inline]
    fn write_to_buffer<B: BufferMut>(&self, buffer: &mut B) {
        buffer.write::<u8, E>(&u8::from(*self));
    }
}

impl<E: Endian> Instruct<E> for Bytes {
    #[inline]
    fn write_to_buffer<B: BufferMut>(&self, buffer: &mut B) {
        buffer.extend_from_slice(self.as_ref());
    }
}

impl<E: Endian, T: Instruct<E>> Instruct<E> for Vec<T> {
    #[inline]
    fn write_to_buffer<B: BufferMut>(&self, buffer: &mut B) {
        for item in self {
            buffer.write::<T, E>(item);
        }
    }
}

impl<E, T1, T2> Instruct<E> for (T1, T2)
where
    E: Endian,
    T1: Instruct<E>,
    T2: Instruct<E>,
{
    #[inline]
    fn write_to_buffer<B: BufferMut>(&self, buffer: &mut B) {
        buffer.write::<T1, E>(&self.0);
        buffer.write::<T2, E>(&self.1);
    }
}

impl<E, T1, T2, T3> Instruct<E> for (T1, T2, T3)
where
    E: Endian,
    T1: Instruct<E>,
    T2: Instruct<E>,
    T3: Instruct<E>,
{
    #[inline]
    fn write_to_buffer<B: BufferMut>(&self, buffer: &mut B) {
        buffer.write::<T1, E>(&self.0);
        buffer.write::<T2, E>(&self.1);
        buffer.write::<T3, E>(&self.2);
    }
}

impl<E, T1, T2, T3, T4> Instruct<E> for (T1, T2, T3, T4)
where
    E: Endian,
    T1: Instruct<E>,
    T2: Instruct<E>,
    T3: Instruct<E>,
    T4: Instruct<E>,
{
    #[inline]
    fn write_to_buffer<B: BufferMut>(&self, buffer: &mut B) {
        buffer.write::<T1, E>(&self.0);
        buffer.write::<T2, E>(&self.1);
        buffer.write::<T3, E>(&self.2);
        buffer.write::<T4, E>(&self.3);
    }
}

macro_rules! impl_prim_pack {
    ($($t:ident),+) => {
        $(
            impl<E: Endian> Instruct<E> for $t {
                #[inline]
                fn write_to_buffer<B: BufferMut + ?Sized>(&self, buffer: &mut B) {
                    buffer.extend_from_slice(&<E as WritePrimitive>::$t(*self));
                }
            }
        )*
    }
}

impl_prim_pack!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64);

macro_rules! gen_endian_trait {
    ($($ty:ident),+) => {

        pub trait WritePrimitive {
            $(
                fn $ty(prim: $ty) -> [u8; core::mem::size_of::<$ty>()];
            )*
        }

        impl WritePrimitive for crate::LittleEndian {
            $(
                #[inline(always)]
                fn $ty(prim: $ty) -> [u8; core::mem::size_of::<$ty>()] {
                    prim.to_le_bytes()
                }
            )*
        }

        impl WritePrimitive for crate::BigEndian {
            $(
                #[inline(always)]
                fn $ty(prim: $ty) -> [u8; core::mem::size_of::<$ty>()] {
                    prim.to_be_bytes()
                }
            )*
        }
    }
}

gen_endian_trait!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64);
