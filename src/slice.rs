use crate::{ConstWriterAdapter, ConstWrite, ConstWriterAdapterCreate};

/// Wrapper for `&mut [u8]`. Advances wrapped slice reference on drop.
/// pub user is not intended
/// ```
/// use const_writer::{ConstWriterAdapter, ConstWriterAdapterCreate};
/// use const_writer::slice::SliceWriterAdapter;
///
/// let mut buf = [0u8; 20];
/// let mut ref_buf = &mut buf as &mut [u8];
/// unsafe {
///     let mut adapter = SliceWriterAdapter::new::<20>(&mut ref_buf); // checks slice len to be > 20
///     adapter
///         .write(&[1u8; 2])
///         .write(&[2u8; 4]); // `ref_buf` is unchanged, but inner pointer is advanced
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

unsafe impl<'a, 'inner> ConstWriterAdapterCreate<'a, &'inner mut [u8]> for SliceWriterAdapter<'a, 'inner> {
    unsafe fn new<const N: usize>(slice: &'a mut &'inner mut [u8]) -> Self {
        assert!(
            slice.len() >= N,
            "slice too short: {} < {}",
            slice.len(),
            N
        );
        let ptr = core::mem::transmute::<_, *mut u8>(slice.as_mut_ptr());
        Self {
            slice,
            ptr
        }
    }
}

impl<'a, 'inner> ConstWriterAdapter for SliceWriterAdapter<'a, 'inner> {
    // Because we have exclusive access to slice pointer we can wait with it's modification until adapter is dropped
    unsafe fn write<const N: usize>(mut self, value: &[u8; N]) -> Self {
        core::ptr::copy_nonoverlapping(value.as_ptr(), self.ptr, N);
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
        unsafe {
            let diff = self.ptr.offset_from(self.slice.as_ptr()) as usize;
            *self.slice = core::slice::from_raw_parts_mut(self.ptr, self.slice.len() - diff);
        }
    }
}

impl<'a, 'inner> ConstWrite<'a, SliceWriterAdapter<'a, 'inner>> for &'inner mut [u8] {}



#[cfg(test)]
mod tests {
    extern crate test;
    use test::Bencher;

    use crate::ConstWrite;
    #[test]
    fn slice_write() {
        let mut buff = [0u8; 10];
        buff.as_mut().const_writer::<10>()
            .write_u32_le(34)
            .write_u16_le(3)
            .write_u16_le(4)
            .write_u16_le(5);
        assert_eq!(buff, [34, 0, 0, 0, 3, 0, 4, 0, 5, 0]);
    }

    #[bench]
    fn bench_const_writer_le(b: &mut Bencher) {
        let mut buff = [0u8; 32];
        b.iter(|| {
            let mut ref_buff = buff.as_mut() as &mut [u8];
            ref_buff.const_writer::<31>()
                .write_u8_le(0x01)
                .write_u16_le(0x0203)
                .write_u32_le(0x04050607)
                .write_u64_le(0x08090A0B0C0D0E0F)
                .write_u128_le(0x101112131415161718191A1B1C1D1E1F);
        });
    }

    #[bench]
    fn bench_bytes_le(b: &mut Bencher) {
        use bytes::BufMut;
        let mut buff = [0u8; 32];
        b.iter(|| {
            let mut ref_buff = buff.as_mut() as &mut [u8];
            ref_buff.put_u8(0x01);
            ref_buff.put_u16_le(0x0203);
            ref_buff.put_u32_le(0x04050607);
            ref_buff.put_u64_le(0x08090A0B0C0D0E0F);
            ref_buff.put_u128_le(0x101112131415161718191A1B1C1D1E1F);
        });
    }

    #[bench]
    fn bench_const_writer_be(b: &mut Bencher) {
        let mut buff = [0u8; 32];
        b.iter(|| {
            let mut ref_buff = buff.as_mut() as &mut [u8];
            ref_buff.const_writer::<31>()
                .write_u8_be(0x01)
                .write_u16_be(0x0203)
                .write_u32_be(0x04050607)
                .write_u64_be(0x08090A0B0C0D0E0F)
                .write_u128_be(0x101112131415161718191A1B1C1D1E1F);
            assert_eq!(ref_buff.len(), 1);
        });
    }

    #[bench]
    fn bench_bytes_be(b: &mut Bencher) {
        use bytes::BufMut;
        let mut buff = [0u8; 32];
        b.iter(|| {
            let mut ref_buff = buff.as_mut() as &mut [u8];
            ref_buff.put_u8(0x01);
            ref_buff.put_u16(0x0203);
            ref_buff.put_u32(0x04050607);
            ref_buff.put_u64(0x08090A0B0C0D0E0F);
            ref_buff.put_u128(0x101112131415161718191A1B1C1D1E1F);
            assert_eq!(ref_buff.len(), 1);
        });
    }
}