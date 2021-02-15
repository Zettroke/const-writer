#![feature(test)]
use crate::{ConstWriter, ConstWriterAdapter, ConstWrite};

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

impl<'a> ConstWriterAdapter for VecWriterAdapter<'a> {
    unsafe fn write<const N: usize>(mut self, value: &[u8; N]) -> Self {
        std::ptr::copy_nonoverlapping(value.as_ptr(), self.ptr, N);
        self.ptr = self.ptr.add(N);
        self
    }

    unsafe fn grow<const M: usize>(mut self) -> Self {
        let written_bytes = self.ptr.offset_from(self.vec.as_ptr()) as usize;
        self.vec.reserve(written_bytes + M);
        // vec.reserve() can move inner buffer so we update our pointer
        self.ptr = std::mem::transmute::<_, *mut u8>(self.vec.as_mut_ptr()).add(written_bytes);
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

#[cfg(test)]
mod tests {
    extern crate test;
    use test::Bencher;
    use crate::ConstWrite;
    #[test]
    fn vec_write_le() {
        let mut vec = vec![];

        vec.const_writer::<31>()
            .write_u8_le(0x01)
            .write_u16_le(0x0203)
            .write_u32_le(0x04050607)
            .write_u64_le(0x08090A0B0C0D0E0F)
            .write_u128_le(0x101112131415161718191A1B1C1D1E1F);

        assert_eq!(&vec, &[
            0x01,
            0x03, 0x02,
            0x07, 0x06, 0x05, 0x04,
            0x0F, 0x0E, 0x0D, 0x0C, 0x0B, 0x0A, 0x09, 0x08,
            0x1F, 0x1E, 0x1D, 0x1C, 0x1B, 0x1A, 0x19, 0x18, 0x17, 0x16, 0x15, 0x14, 0x13, 0x12, 0x11, 0x10,
        ]);
    }

    #[test]
    fn vec_write_be() {
        let mut vec = vec![];

        vec.const_writer::<31>()
            .write_u8_be(0x01)
            .write_u16_be(0x0203)
            .write_u32_be(0x04050607)
            .write_u64_be(0x08090A0B0C0D0E0F)
            .write_u128_be(0x101112131415161718191A1B1C1D1E1F);

        assert_eq!(&vec, &[
            0x01,
            0x02, 0x03,
            0x04, 0x05, 0x06, 0x07,
            0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
            0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F
        ]);
    }

    #[test]
    fn vec_write_grow() {
        let mut vec = vec![];

        vec.const_writer::<5>()
            .write_u32_le(123)
            .write_u8_le(1)
            .convert::<10>()
            .write_u32_le(124)
            .write_u8_le(24).remaining();

        assert_eq!(&vec, &[123, 0, 0, 0, 1, 124, 0, 0, 0, 24]);
    }

    #[bench]
    fn bench_bytes_vec(b: &mut Bencher) {
        use bytes::BufMut;
        b.iter(|| {
            let mut vec = Vec::with_capacity(31);
            vec.put_u8(0x01);
            vec.put_u16(0x0203);
            vec.put_u32(0x04050607);
            vec.put_u64(0x08090A0B0C0D0E0F);
            vec.put_u128(0x101112131415161718191A1B1C1D1E1F);
            assert_eq!(vec.len(), 31);
        });
    }

    #[bench]
    fn bench_const_writer_vec(b: &mut Bencher) {
        b.iter(|| {
            let mut vec = Vec::with_capacity(31);
            vec.const_writer::<31>()
                .write_u8_be(0x01)
                .write_u16_be(0x0203)
                .write_u32_be(0x04050607)
                .write_u64_be(0x08090A0B0C0D0E0F)
                .write_u128_be(0x101112131415161718191A1B1C1D1E1F);
            assert_eq!(vec.len(), 31);
        });
    }
}