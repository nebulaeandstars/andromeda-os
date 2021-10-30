#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(andromeda_os::test_runner)]
#![reexport_test_harness_main = "run_test"]

use core::panic::PanicInfo;

use andromeda_os::{halt, println, vga};

bootloader::entry_point!(test_kernel_start);
fn test_kernel_start(boot_info: &'static bootloader::BootInfo) -> ! {
    run_test();
    halt()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    andromeda_os::test_panic_handler(info)
}

#[test_case]
fn test_println() {
    println!("test_println output");
}
