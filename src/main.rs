#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(andromeda_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use andromeda_os::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    andromeda_os::init();

    let a = 4;
    let b = 17;

    println!("Hello World!");
    println!("{} + {} = {}", a, b, a + b);

    #[cfg(test)]
    test_main();

    loop {}
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    andromeda_os::test_panic_handler(info)
}
