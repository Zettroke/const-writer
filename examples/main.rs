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
    println!("{:?}", buff);

    let mut vec = Vec::new();
    write_struct(vec.const_writer());
    write_struct(vec.const_writer());
    vec.const_writer::<24>().write_slice(&[11u8; 24]);

    write_conditional_len(vec.const_writer(), false);
    write_conditional_len(vec.const_writer(), true);


    println!("{:?}", vec);
    println!("{:?}", vec.capacity());

    let mut vec = vec![];

    vec.const_writer::<5>()
        .write_u32_le(123)
        .write_u8_le(1)
        .convert::<10>()
        .write_u32_le(124)
        .write_u8_le(24);

    write_conditional_len(vec.const_writer(), true);

    println!("{:?}", vec);
    println!("{:?}", vec.capacity());
}

fn write_conditional_len<T: ConstWriterAdapter>(writer: ConstWriter<T, 32>, flag: bool) {
    let writer = writer.write_u32_le(24);

    let writer = if flag {
        writer
            .write_u64_le(32)
            .write_u128_le(64)
    } else {
        writer.convert() // rust infer same len as top branch
    };

    assert_eq!(writer.remaining(), 4);
    writer.write_u32_le(48);
}