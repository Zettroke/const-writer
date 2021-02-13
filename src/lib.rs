#![feature(const_generics)]
#![feature(const_evaluatable_checked)]
#![feature(type_name_of_val)]
#![allow(incomplete_features)]

use std::marker::PhantomData;

pub trait IsTrue {}
pub trait IsFalse {}

pub struct Assert<const CHECK: bool> {}

impl IsTrue for Assert<true> {}
impl IsFalse for Assert<false> {}

pub struct SliceConstWriter<'a, const N: usize> {
    ptr: *mut u8,
    _marker: PhantomData<&'a ()>
}

macro_rules! advance_writer {
    ($self:ident, $n:expr) => {
        ConstWrite {
            ptr: $self.ptr.add($n),
            _marker: PhantomData
        }
    }
}

macro_rules! implement_write {
    ($writer:ident, $name:ident, $type:ty, $size:expr) => {
        impl<'a, const N: usize> $writer<'a, {N}> where Assert::<{N >= $size}>: IsTrue {
            pub fn $name(self, value: $type) -> $writer<'a, {N - $size}> {
                unsafe {
                    std::ptr::copy_nonoverlapping(value.to_le_bytes().as_ptr(),self.ptr, $size);
                    $writer {
                        ptr: self.ptr.add($size),
                        _marker: PhantomData
                    }
                }
            }
        }
    }
}
impl<'a, const N: usize> SliceConstWriter<'a, {N}> {
    pub fn from_slice(value: &'a mut [u8]) -> Self {
        if value.len() >= N {
            Self {
                ptr: value.as_mut_ptr(),
                _marker: PhantomData
            }
        } else {
            panic!();
        }
    }
}

pub trait ConstWriter<const N: usize> {
    fn write_u8(self, value: u8) -> Self;
}

// impl<'a, const N: usize> ConstWriter<{N}> for SliceConstWriter<'a, {N}> {
//     fn write_u8(self, value: u8) -> SliceConstWriter<'a, {N - 1}> {
//         unsafe {
//             std::ptr::copy_nonoverlapping(value.to_le_bytes().as_ptr(),self.ptr, 1);
//             SliceConstWriter {
//                 ptr: self.ptr.add(1),
//                 _marker: PhantomData
//             }
//         }
//     }
// }

implement_write!(SliceConstWriter, write_u16, u16, 2);
implement_write!(SliceConstWriter, write_u32, u32, 4);

pub trait ConstWrite {
    fn const_writer<const N: usize>(&mut self) -> SliceConstWriter<{N}>;
}

impl ConstWrite for [u8] {
    fn const_writer<const N: usize>(&mut self) -> SliceConstWriter<{ N }> {
        assert!(
            self.len() >= N,
            "slice too short: {} < {}",
            self.len(),
            N
        );
        SliceConstWriter {
            ptr: self.as_mut_ptr(),
            _marker: PhantomData
        }
    }
}





#[cfg(test)]
mod tests {
    use crate::{SliceConstWriter, ConstWrite};

    #[test]
    fn it_works() {
        let mut buff = [0u8; 128];
        let mut a = buff.const_writer::<10>();

        let a = a.write_u32(34).write_u16(3).write_u16(4).write_u16(5);

        std::any::type_name_of_val(&a);
    }
}
