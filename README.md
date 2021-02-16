Const Writer
=============
[![Documentation](https://docs.rs/const-writer/badge.svg)](https://docs.rs/const-writer)
[![Latest version](https://img.shields.io/crates/v/const-writer.svg)](https://crates.io/crates/const-writer)

####[Documentation](https://docs.rs/const-writer/badge.svg)

Compile time verified byte writer for slice and vector.

### Install
```toml
[dependencies]
const-writer = "0.1.0"
```
### Usage example
 ```rust
use const_writer::ConstWrite;
fn main() {
    let mut vec = vec![];
    let writer = vec.const_writer::<10>() // reserve 10 bytes in vec
        .write_u32_le(12)  // no runtime checks
        .write_u32_le(34); // no runtime checks

    assert_eq!(writer.remaining(), 2);
    assert_eq!(vec.len(), 8);
    assert_eq!(&vec[0..8], &[12, 0, 0, 0, 34, 0, 0, 0]);
}
 ```

```rust
use const_writer::ConstWrite;
fn main() {
    let mut buff = [0u8; 1024];
    buff.as_mut().const_writer::<10>()
        .write_slice(&[1, 2, 3, 4, 5, 6])
        .write_u64_le(111); // compile error.
}
```

```rust
 use const_writer::{ConstWriterAdapter, ConstWriter, ConstWrite};
 fn main() {
 // write 10 bytes
 fn write_struct<T: ConstWriterAdapter>(writer: ConstWriter<T, 10>) {
     writer
         .write_u16_le(34)
         .write_u16_le(2)
         .write_u16_le(3)
         .write_u16_le(4)
         .write_u16_le(5);
 }

 let mut buff = [0u8; 16];
 write_struct(buff.as_mut().const_writer());
 assert_eq!(buff, [34, 0, 2, 0, 3, 0, 4, 0, 5, 0, 0, 0, 0, 0, 0, 0]);
}
 ```

### Assembly
It generates super optimal assembly with pretty rust syntax
##### Rust:
```rust
// we return modified reference so compiler won't optimise away reference manipulation
pub fn write(mut ref_buff: &mut [u8]) -> &mut [u8] {
    use crate::ConstWrite;
    ref_buff.const_writer::<31>()
        .write_u8_le(0x01)
        .write_u16_le(0x0203)
        .write_u32_le(0x04050607)
        .write_u64_le(0x08090A0B0C0D0E0F)
        .write_u128_le(0x101112131415161718191A1B1C1D1E1F);

    ref_buff
}
```
##### Assembly:
Assembly consists only one comparison
```asm
write:
	subq	$88, %rsp
	movq	%rsi, %rdx
	cmpq	$30, %rsi
	jbe	.LBB2_1 # panic
	movq	%rdi, %rax
	movb	$1, (%rdi)
	movw	$515, 1(%rdi)                   # imm = 0x203
	movl	$67438087, 3(%rdi)              # imm = 0x4050607
	movabsq	$579005069656919567, %rcx       # imm = 0x8090A0B0C0D0E0F
	movq	%rcx, 7(%rdi)
	movabsq	$1157726452361532951, %rcx      # imm = 0x1011121314151617
	movq	%rcx, 23(%rdi)
	movabsq	$1736447835066146335, %rcx      # imm = 0x18191A1B1C1D1E1F
	movq	%rcx, 15(%rdi)
	addq	$31, %rax
	addq	$-31, %rdx
	addq	$88, %rsp
	retq
```
And sometimes rustc can generate really pretty assembly :heart_eyes:

##### Rust:
```rust
pub fn write_struct<T: ConstWriterAdapter>(writer: ConstWriter<T, 12>) -> ConstWriter<T, 0> {
    writer.write_u32_le(34).write_u16_le(2).write_u16_le(3).write_u16_le(4).write_u16_le(5)
}
```
##### Assembly:
```asm
write_struct:
	movq	%rdi, %rax
	movl	$34, (%rsi)
	movabsq	$1407392063619074, %rcx         # imm = 0x5000400030002 <- OMG
	movq	%rcx, 4(%rsi)
	leaq	12(%rsi), %rdx
	retq
```

### Features
* Support `no_std`