
pub struct LittleEndian;
pub struct BigEndian;

#[cfg(target_endian = "little")]
pub type NativeEndian = LittleEndian;

#[cfg(target_endian = "big")]
pub type NativeEndian = BigEndian;

pub type NetworkEndian = BigEndian;

macro_rules! gen_endian_trait {
    ($($ty:ident),+) => {
        pub trait Endian {
            $(
                fn $ty(bytes: [u8; core::mem::size_of::<$ty>()]) -> $ty;
            )*
        }

        impl Endian for LittleEndian {
            $(
                fn $ty(bytes: [u8; core::mem::size_of::<$ty>()]) -> $ty {
                    $ty::from_le_bytes(bytes)
                }
            )*
        }

        impl Endian for BigEndian {
            $(
                fn $ty(bytes: [u8; core::mem::size_of::<$ty>()]) -> $ty {
                    $ty::from_be_bytes(bytes)
                }
            )*
        }
    }
}

gen_endian_trait!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);