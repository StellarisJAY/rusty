use core::panic::PanicInfo;
use crate::println;
use crate::sys_exit;
#[panic_handler]
fn panic(info: &PanicInfo)->!{
    if let Some(location) = info.location() {
        println!(
            "Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        println!("Panicked: {}", info.message().unwrap());
    }
    sys_exit(0);
    panic!("not reachable")
}