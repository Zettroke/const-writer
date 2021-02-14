#![feature(const_generics)]
#![feature(const_evaluatable_checked)]
#![feature(type_name_of_val)]
#![allow(incomplete_features)]


use const_writer::{ConstWriter, ConstWrite, ConstWriterAdapter};

pub fn write_struct<T: ConstWriterAdapter>(writer: ConstWriter<T, 10>) -> ConstWriter<T, 0> {
    writer.write_u16_le(34).write_u16_le(2).write_u16_le(3).write_u16_le(4).write_u16_le(5)
}

fn main() {
    let mut buff = [0u8; 24];
    {
        let mut ref_buff = buff.as_mut();

        let writer = ref_buff.const_writer::<20>();

        // decrease writer size without check
        let writer = write_struct(writer.convert());
        // increase writer size with check
        write_struct(writer.convert());
    }
    let mut vec = Vec::new();
    write_struct(vec.const_writer());
    write_struct(vec.const_writer());
    vec.const_writer::<24>().write_slice(&[11u8; 24]);


    println!("{:?}", buff);
    println!("{:?}", vec);
    println!("{:?}", vec.capacity());
}