#![feature(const_generics)]
#![feature(const_evaluatable_checked)]
#![feature(type_name_of_val)]
#![allow(incomplete_features)]
#![feature(test)]

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

pub trait IsTrue {}
pub trait IsFalse {}

pub struct Assert<const CHECK: bool> {}

impl IsTrue for Assert<true> {}
impl IsFalse for Assert<false> {}

/// Implementation must check capacity of inner buffer on creation of struct.
/// And user must not exceed this capacity
///
/// Because const generics expressions in traits works really bad,
/// this adapter doesn't has generic len param, so write is basically unchecked write to array.
/// This adapter must be used within [`ConstWriter`] because it holds and tracks buffer length
pub trait ConstWriterAdapter {
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

/// Wrapper for `&mut [u8]`. Advances wrapped slice reference on drop.
/// pub user is not intended
/// ```
/// use crate::const_writer::SliceWriterAdapter;
/// use const_writer::ConstWriterAdapter;
///
/// let mut buf = [0u8; 20];
/// let mut ref_buf = &mut buf as &mut [u8];
/// unsafe {
///     let mut adapter = SliceWriterAdapter::new::<20>(&mut ref_buf); // checks slice len to be > 20
///     adapter
///         .write(&[1u8; 2])
///         .write(&[2u8; 4]); // `buf` is unchanged, but inner pointer is advanced
/// };
/// //after adapter dropped pointer is advanced
/// assert_eq!(ref_buf.len(), 14);
/// assert_eq!(buf[..6], [1, 1, 2, 2, 2, 2])
/// ```
pub struct SliceWriterAdapter<'a, 'inner> {
    /// original slice
    slice: &'a mut &'inner mut [u8],
    /// ptr to slice data
    ptr: *mut u8

}

impl<'a, 'inner> SliceWriterAdapter<'a, 'inner> {
    /// Creates adapter from slice, checks it's length
    pub unsafe fn new<const N: usize>(slice: &'a mut &'inner mut [u8]) -> Self {
        assert!(
            slice.len() >= N,
            "slice too short: {} < {}",
            slice.len(),
            N
        );
        let ptr = std::mem::transmute::<_, *mut u8>(slice.as_mut_ptr());
        Self {
            slice,
            ptr
        }
    }
}

impl<'a, 'inner> ConstWriterAdapter for SliceWriterAdapter<'a, 'inner> {
    // Because we have exclusive access to slice pointer we can wait with it's modification until adapter is dropped
    unsafe fn write<const N: usize>(mut self, value: &[u8; N]) -> Self {
        std::ptr::copy_nonoverlapping(value.as_ptr(), self.ptr, N);
        self.ptr = self.ptr.add(N);
        self
    }

    unsafe fn grow<const M: usize>(self) -> Self {
        let diff = self.ptr.offset_from(self.slice.as_ptr()) as usize;
        assert!(
            M <= self.slice.len() - diff,
            "remaining slice too short to grow: {} < {}",
            self.slice.len() - diff,
            M
        );
        self
    }
}

impl<'a, 'inner> Drop for SliceWriterAdapter<'a, 'inner> {
    /// When dropping adapter we advancing slice pointer
    fn drop(&mut self) {
        println!("drop {}", std::any::type_name_of_val(self));
        unsafe {
            let diff = self.ptr.offset_from(self.slice.as_ptr()) as usize;
            *self.slice = std::slice::from_raw_parts_mut(self.ptr, self.slice.len() - diff);
        }
    }
}

pub struct VecWriterAdapter<'a> {
    vec: &'a mut Vec<u8>,
    ptr: *mut u8
}

impl<'a> VecWriterAdapter<'a> {
    /// Creates adapter from slice, reserve enough bytes
    unsafe fn from<const N: usize>(value: &'a mut Vec<u8>) -> Self {
        value.reserve(N);
        let ptr = std::mem::transmute(value.as_mut_ptr().add(value.len()));
        Self {
            vec: value,
            ptr
        }
    }
}

impl<'a> ConstWriterAdapter for VecWriterAdapter<'a> {
    unsafe fn write<const N: usize>(mut self, value: &[u8; N]) -> Self {
        std::ptr::copy_nonoverlapping(value.as_ptr(), self.ptr, N);
        self.ptr = self.ptr.add(N);
        self
    }

    unsafe fn grow<const M: usize>(self) -> Self {
        let written_bytes = self.ptr.offset_from(self.vec.as_ptr()) as usize;
        self.vec.reserve(written_bytes - self.vec.len() + M);
        self
    }
}

impl<'a> Drop for VecWriterAdapter<'a> {
    /// When dropping adapter we advancing vector
    fn drop(&mut self) {
        unsafe {
            let new_len = self.ptr.offset_from(self.vec.as_ptr()) as usize;
            self.vec.set_len(new_len);
        }
    }
}


pub struct ConstWriter<T: ConstWriterAdapter, const N: usize> {
    writer_adapter: T
}

macro_rules! implement_write {
    ($name:ident, $type:ty, $endian:ident) => {
        pub fn $name(self, value: $type) ->ConstWriter<T, {N - std::mem::size_of::<$type>()}> {
            unsafe {
                ConstWriter {
                    writer_adapter: self.writer_adapter.write(&value.$endian())
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
    /// If `M` > `N` there is check for slices and reserve for vectors
    pub fn convert<const M: usize>(self) -> ConstWriter<T, {M}> {
        if M <= N { // shrink
            ConstWriter {
                writer_adapter: self.writer_adapter
            }
        } else {
            unsafe {
                ConstWriter { // grow
                    writer_adapter: self.writer_adapter.grow::<{M}>()
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

    pub fn remaining(&self) -> usize {
        N
    }
}

impl<T: ConstWriterAdapter, const N: usize> ConstWriter<T, {N}> {
    pub fn write_slice<const M: usize>(self, value: &[u8; M]) -> ConstWriter<T, { N-M }>/* where Assert::<{N >= M}>: IsTrue*/ {
        unsafe {
            ConstWriter {
                writer_adapter: self.writer_adapter.write(value)
            }
        }
    }
}

pub trait ConstWrite<'a, T: ConstWriterAdapter> {
    /// Get [`ConstWriter`] to write `N` bytes. Performs checks/allocations so at least `N` bytes
    fn const_writer<const N: usize>(&'a mut self) -> ConstWriter<T, {N}>;
}

impl<'a, 'inner> ConstWrite<'a, SliceWriterAdapter<'a, 'inner>> for &'inner mut [u8] {
    /// Get const writer for `N` bytes. Panics if slice too short
    fn const_writer<const N: usize>(&'a mut self) -> ConstWriter<SliceWriterAdapter<'a, 'inner>, { N }> {
        // `SliceWriterAdapter::from` checks that slice len greater or equal than `N`.
        // Because we ensure that ConstWriter never writes more than `N` bytes
        unsafe {
            ConstWriter {
                writer_adapter: SliceWriterAdapter::new::<{ N }>(self)
            }
        }
    }
}

impl<'a> ConstWrite<'a, VecWriterAdapter<'a>> for Vec<u8> {
    /// Get const writer for `N` bytes. Reserve `N` bytes in vector
    fn const_writer<const N: usize>(&'a mut self) -> ConstWriter<VecWriterAdapter<'a>, { N }> {
        // `VecWriterAdapter::from` ensure that vec can hold `N` bytes.
        // Because we ensure that ConstWriter never writes more than `N` bytes
        unsafe {
            ConstWriter {
                writer_adapter: VecWriterAdapter::from::<{ N }>(self)
            }
        }
    }
}

#[cfg(feature = "bytes-writer")]
mod bytes_writer {
    use bytes::BytesMut;
    use crate::ConstWriterAdapter;

    pub struct BytesWriteAdapter<'a> {
        bytes: &'a mut BytesMut,
        ptr: *mut u8
    }

    impl<'a> BytesWriteAdapter<'a> {
        /// Creates adapter from slice, reserve enough bytes
        unsafe fn from<const N: usize>(value: &'a mut BytesMut) -> Self {
            value.reserve(N);
            let ptr = std::mem::transmute(value.as_mut_ptr().add(value.len()));
            Self {
                bytes: value,
                ptr
            }
        }
    }

    // impl<'a> ConstWriterAdapter for BytesWriteAdapter<'a> {
    //     unsafe fn write<const N: usize>(self, value: &[U8; N]) -> Self {
    //
    //     }
    // }
}

pub fn add_two(a: i32) -> i32 {
    a.pow(123)
}

#[cfg(test)]
mod tests {
    extern crate test;
    use crate::{ConstWrite};
    use test::Bencher;

    #[test]
    fn slice_write() {
        // println!("it_works");
        let mut buff = [0u8; 10];
        let mut ref_buff = &mut buff as &mut [u8];
        ref_buff.const_writer::<10>().write_u32_le(34).write_u16_le(3).write_u16_le(4).write_u16_le(5);
        assert_eq!(buff, [34, 0, 0, 0, 3, 0, 4, 0, 5, 0]);
    }

    // #[cfg(feature = "bytes-writer")]
    // #[test]
    // fn compile_test() {
    //     let builder = trybuild::TestCases::new();
    //     builder.compile_fail("compile_tests/overflow.rs");
    // }

    #[bench]
    fn bench_add_two(b: &mut Bencher) {
        let mut buff = [0u8; 25];
        b.iter(|| {
            let mut ref_buff = &mut buff as &mut [u8];
            ref_buff.const_writer::<10>()
                .write_u16_le(34)
                .write_u16_le(2)
                .write_u16_le(3)
                .write_u16_le(4)
                .write_u16_le(5);
            assert_eq!(buff[..10], [34, 0, 2, 0, 3, 0, 4, 0, 5, 0]);
        });
    }
    #[bench]
    fn bench_add_two2(b: &mut Bencher) {
        use bytes::BufMut;
        let mut buff = [0u8; 25];
        b.iter(|| {
            let mut ref_buff = &mut buff as &mut [u8];
            ref_buff.put_u16_le(34);
            ref_buff.put_u16_le(2);
            ref_buff.put_u16_le(3);
            ref_buff.put_u16_le(4);
            ref_buff.put_u16_le(5);
            assert_eq!(buff[..10], [34, 0, 2, 0, 3, 0, 4, 0, 5, 0]);
        });
    }
}
