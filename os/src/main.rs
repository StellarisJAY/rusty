#![no_std]
#![no_main]

mod lang_items;

use core::arch::global_asm;

// entry point: entry.asm
global_asm!(include_str!("entry.asm"));
