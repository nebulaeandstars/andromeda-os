#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(andromeda_os::test_runner)]
#![reexport_test_harness_main = "run_test"]

extern crate alloc;

use core::panic::PanicInfo;

use andromeda_os::task::{keyboard, Executor};
use andromeda_os::vga::Color::*;
use andromeda_os::{halt, println, vga};
use bootloader::BootInfo;

fn main() {
    vga::with_color(LightCyan, Black, || {
        println!("Hello, world!\n");
    });

    let mut s = alloc::string::String::from("This is a String on the heap!");
    println!("{:?}", s);
    s.push_str(" It can be expanded.");
    println!("{:?}", s);

    let mut executor = Executor::new(100);
    executor.spawn(keyboard::print_keypresses());
    executor.run()
}

bootloader::entry_point!(kernel_start);
fn kernel_start(boot_info: &'static BootInfo) -> ! {
    andromeda_os::init(boot_info);

    #[cfg(test)]
    run_test();

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
