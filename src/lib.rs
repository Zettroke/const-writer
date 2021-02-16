#![feature(const_generics)]
#![feature(const_evaluatable_checked)]
#![feature(type_name_of_val)]
#![allow(incomplete_features)]
#![feature(test)]

#![cfg_attr(not(feature = "std"), no_std)]

//! Provides [`ConstWriter`] abstraction to write constant amount of bytes with compile time checks
//!
//! Result of fun experiment with `const_generics` and `const_evaluatable_checked` features
//!
//! ```
//! use const_writer::ConstWrite;
//!
//! let mut vec = vec![];
//! {
//!     let writer = vec.const_writer::<10>() // reserve 10 bytes in vec
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

/// Trait for creating `ConstWriterAdapter`
/// Creation moved to separate trait to omit lifetime parameter on ConstWriter
pub unsafe trait ConstWriterAdapterCreate<'a, T: ?Sized> {
    /// # Safety
    /// You must ensure that underlying buffer have space for at least `N` bytes.
    unsafe fn new<const N: usize>(buff: &'a mut T) -> Self;
}

/// Source of all performance of crate. Provide unsafe interface to underlying buffer.
///
/// Because const generics expressions in traits works really bad,
/// this adapter doesn't has generic len param, so write is basically unchecked write to array.
/// This adapter must be used within [`ConstWriter`] because it holds and tracks buffer length
pub trait ConstWriterAdapter {
    /// Write bytes and advances inner buffer
    ///
    /// # Safety
    /// Unsafe because with current `const_generics` and `const_evaluatable_checked` we can't
    /// define trait which returns self with calculated const generic param.
    ///
    /// You should make sure that in total you advance less or equal than `N` bytes
    unsafe fn write<const N: usize>(self, value: &[u8; N]) -> Self;

    /// Ensures that underlying buffer have space for `M` additional bytes
    /// # Example
    /// If 5 bytes were written to buffer, then `grow::<10>()` will ensure that
    /// underlying buffer have capacity at least 15
    unsafe fn grow<const M: usize>(self) -> Self;
}

pub mod slice;

#[cfg(any(feature = "std", feature = "alloc"))]
pub mod vec;

///
/// Writer that keeping track of space left using const_generic params.
///
pub struct ConstWriter<T: ConstWriterAdapter, const N: usize> {
    writer_adapter: T,
}

macro_rules! implement_write {
    ($name:ident, $type:ty, $endian:ident) => {
        pub fn $name(self, value: $type) ->ConstWriter<T, {N - core::mem::size_of::<$type>()}> {
            unsafe {
                ConstWriter {
                    writer_adapter: self.writer_adapter.write(&value.$endian()),
                }
            }
        }

    }
}

impl<T: ConstWriterAdapter, const N: usize> ConstWriter<T, {N}> {
    /// Changes length of [`ConstWriter`] to `M`.
    ///
    /// If `M` <= `N` then no checks or allocation invoked
    ///
    /// If `M` > `N` there adapter ensures that underlying buffer have space for `M` more bytes.
    pub fn convert<const M: usize>(self) -> ConstWriter<T, {M}> {
        if M <= N { // shrink
            ConstWriter {
                writer_adapter: self.writer_adapter,

            }
        } else {
            unsafe {
                ConstWriter { // grow
                    writer_adapter: self.writer_adapter.grow::<{M}>(),

                }
            }
        }
    }
}

impl<T: ConstWriterAdapter, const N: usize> ConstWriter<T, {N}> {
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

    /// Helper to access const_generic param
    pub fn remaining(&self) -> usize {
        N
    }
}

impl<T: ConstWriterAdapter, const N: usize> ConstWriter<T, {N}> {
    pub fn write_slice<const M: usize>(self, value: &[u8; M]) -> ConstWriter<T, { N-M }> {
        unsafe {
            ConstWriter {
                writer_adapter: self.writer_adapter.write(value),

            }
        }
    }
}

/// Get [`ConstWriter`] for given type
pub trait ConstWrite<'a, T: ConstWriterAdapter + ConstWriterAdapterCreate<'a, Self>> {
    /// Get [`ConstWriter`] to write `N` bytes.
    ///
    /// Because contract on `ConstWriterAdapterCreate::new` we can be sure that underlying buffer
    /// can accept at least `N` bytes. And because write methods reduces `N` as they write to buffer
    /// we can be sure that code which writes more than`N` bytes wont compile
    /// (N is usize so negative value will be compile error)
    fn const_writer<const N: usize>(&'a mut self) -> ConstWriter<T, {N}> {
        unsafe {
            ConstWriter {
                writer_adapter: T::new::<{ N }>(self)
            }
        }
    }
}

