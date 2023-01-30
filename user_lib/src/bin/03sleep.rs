#![no_std]
#![no_main]

#[macro_use]
extern crate lib_rusty;

#[no_mangle]
fn main() -> i32 {
    println!("Test sleep OK!");
    0
}
