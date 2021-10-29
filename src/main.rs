#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(andromeda_os::test_runner)]
#![reexport_test_harness_main = "run_test"]

use core::panic::PanicInfo;

use andromeda_os::{halt, println};

fn main() {
    println!("Hello, world!");
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    andromeda_os::init();

    #[cfg(test)]
    run_test();
    #[cfg(not(test))]
    main();

    halt()
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    halt()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    andromeda_os::test_panic_handler(info)
}
