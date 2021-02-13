#![feature(const_generics)]
#![feature(const_evaluatable_checked)]
#![feature(type_name_of_val)]



use const_writer::{SliceConstWriter, ConstWrite};

fn main() {
    let mut buff = [0u8; 128];
    let mut a = buff.const_writer::<256>();

    let a = a.write_u16(1).write_u16(2).write_u16(3).write_u16(4).write_u16(5);

    println!("{}", std::any::type_name_of_val(&a));
}