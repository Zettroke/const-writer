#![feature(const_generics)]
#![feature(const_evaluatable_checked)]
#![feature(type_name_of_val)]
#![allow(incomplete_features)]
#![feature(test)]

#![cfg_attr(not(feature = "std"), no_std)]



//! Provides [`ConstWriter`] abstraction to write constant amount of bytes with compile time checks
//!
//! Result of fun experiment with `const_generics` feature
//!
//! ```
//! use const_writer::ConstWrite;
//!
//! let mut vec = vec![];
//! {
//!     let writer = vec.const_writer::<10>() // reserve 8 bytes in vec
//!         .write_u32_le(12)  // no runtime checks
//!         .write_u32_le(34); // no runtime checks
//!
//!     assert_eq!(writer.remaining(), 2);
//! }
//! assert_eq!(vec.len(), 8);
//! assert_eq!(&vec[0..8], &[12, 0, 0, 0, 34, 0, 0, 0]);
//! ```
//!
//! ```compile_fail
//! use const_writer::ConstWrite;
//!
//! let mut buff = [0u8; 1024];
//! buff.as_mut().const_writer::<10>()
//!     .write_slice(&[1, 2, 3, 4, 5, 6])
//!     .write_u64_le(111); // compile error.
//! ```
//!
//! Usage in functions
//! ```
//! use const_writer::{ConstWriterAdapter, ConstWriter, ConstWrite};
//!
//! // write 10 bytes
//! fn write_struct<T: ConstWriterAdapter>(writer: ConstWriter<T, 10>) {
//!     writer
//!         .write_u16_le(34)
//!         .write_u16_le(2)
//!         .write_u16_le(3)
//!         .write_u16_le(4)
//!         .write_u16_le(5);
//! }
//!
//! let mut buff = [0u8; 16];
//! write_struct(buff.as_mut().const_writer());
//! assert_eq!(buff, [34, 0, 2, 0, 3, 0, 4, 0, 5, 0, 0, 0, 0, 0, 0, 0]);
//! ```
//!
//!
//!
// #[cfg(feature = "std")]
// use std::mem;
// #[cfg(not(feature = "std"))]
// use core::mem;

use core::marker::PhantomData;

pub trait IsTrue {}
pub trait IsFalse {}

pub struct Assert<const CHECK: bool> {}

impl IsTrue for Assert<true> {}
impl IsFalse for Assert<false> {}

/// Source of all performance of crate. Provide unsafe interface to underlying buffer.
///
/// Because const generics expressions in traits works really bad,
/// this adapter doesn't has generic len param, so write is basically unchecked write to array.
/// This adapter must be used within [`ConstWriter`] because it holds and tracks buffer length
pub trait ConstWriterAdapter<'a> {
    type Inner;
    unsafe fn new<const N: usize>(v: &'a mut Self::Inner) -> Self;
    /// Advance inner buffer by `value` bytes
    ///
    /// # Safety
    /// Unsafe because with current `const_generics` and `const_evaluatable_checked` we can't
    /// define trait which returns self with calculated const generic param.
    ///
    /// You should make sure that in total you advance less or equal than `N` bytes used in from method.
    unsafe fn write<const N: usize>(self, value: &[u8; N]) -> Self;

    /// Checks if we have enough space to write `M` bytes in underlying buffer
    unsafe fn grow<const M: usize>(self) -> Self;
}

pub mod slice;

#[cfg(any(feature = "std", feature = "alloc"))]
pub mod vec;


pub struct ConstWriter<'a, T: ConstWriterAdapter<'a>, const N: usize> {
    writer_adapter: T,
    _marker: PhantomData<&'a ()>
}

macro_rules! implement_write {
    ($name:ident, $type:ty, $endian:ident) => {
        pub fn $name(self, value: $type) ->ConstWriter<'a, T, {N - core::mem::size_of::<$type>()}> {
            unsafe {
                ConstWriter {
                    writer_adapter: self.writer_adapter.write(&value.$endian()),
                    _marker: PhantomData
                }
            }
        }

    }
}

impl<'a, T: ConstWriterAdapter<'a>, const N: usize> ConstWriter<'a, T, {N}> {
    /// Changes length of [`ConstWriter`] to `M`.
    ///
    /// If `M` <= `N` then no checks or allocation invoked
    ///
    /// If `M` > `N` there is check for slices and reserve for vectors
    pub fn convert<const M: usize>(self) -> ConstWriter<'a, T, {M}> {
        if M <= N { // shrink
            ConstWriter {
                writer_adapter: self.writer_adapter,
                _marker: PhantomData
            }
        } else {
            unsafe {
                ConstWriter { // grow
                    writer_adapter: self.writer_adapter.grow::<{M}>(),
                    _marker: PhantomData
                }
            }
        }
    }
}

// impl<'a, T: ConstWriterAdapter> ConstWrite<'a, T> for T::Inner {
//     fn const_writer<const N: usize>(&'a mut self) -> ConstWriter<T, { N }> {
//         unsafe {
//             ConstWriter {
//                 writer_adapter: T::new::<{ N }>(self)
//             }
//         }
//     }
// }

impl<'a, T: ConstWriterAdapter<'a>, const N: usize> ConstWriter<'a, T, {N}> {
    implement_write!(write_u8_le, u8, to_le_bytes);
    implement_write!(write_u16_le, u16, to_le_bytes);
    implement_write!(write_u32_le, u32, to_le_bytes);
    implement_write!(write_u64_le, u64, to_le_bytes);
    implement_write!(write_u128_le, u128, to_le_bytes);

    implement_write!(write_i8_le, i8, to_le_bytes);
    implement_write!(write_i16_le, i16, to_le_bytes);
    implement_write!(write_i32_le, i32, to_le_bytes);
    implement_write!(write_i64_le, i64, to_le_bytes);
    implement_write!(write_i128_le, i128, to_le_bytes);

    implement_write!(write_u8_be, u8, to_be_bytes);
    implement_write!(write_u16_be, u16, to_be_bytes);
    implement_write!(write_u32_be, u32, to_be_bytes);
    implement_write!(write_u64_be, u64, to_be_bytes);
    implement_write!(write_u128_be, u128, to_be_bytes);

    implement_write!(write_i8_be, i8, to_be_bytes);
    implement_write!(write_i16_be, i16, to_be_bytes);
    implement_write!(write_i32_be, i32, to_be_bytes);
    implement_write!(write_i64_be, i64, to_be_bytes);
    implement_write!(write_i128_be, i128, to_be_bytes);

    implement_write!(write_f32_be, f32, to_be_bytes);
    implement_write!(write_f64_be, f64, to_be_bytes);

    implement_write!(write_f32_le, f32, to_le_bytes);
    implement_write!(write_f64_le, f64, to_le_bytes);

    pub fn remaining(&self) -> usize {
        N
    }
}

impl<'a, T: ConstWriterAdapter<'a>, const N: usize> ConstWriter<'a, T, {N}> {
    pub fn write_slice<const M: usize>(self, value: &[u8; M]) -> ConstWriter<'a, T, { N-M }>/* where Assert::<{N >= M}>: IsTrue*/ {
        unsafe {
            ConstWriter {
                writer_adapter: self.writer_adapter.write(value),
                _marker: PhantomData
            }
        }
    }
}

pub trait ConstWrite<'a, T: ConstWriterAdapter<'a>> {
    /// Get [`ConstWriter`] to write `N` bytes. Performs checks/allocations so at least `N` bytes
    fn const_writer<const N: usize>(&'a mut self) -> ConstWriter<'a, T, {N}>;
}

#[cfg(test)]
mod tests {
    extern crate test;
    use crate::{ConstWrite};
    use test::Bencher;


}
