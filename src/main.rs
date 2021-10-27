#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use andromeda_os::println;
use andromeda_os::vga::VGA_WRITER;

#[no_mangle]
pub extern "C" fn _start() -> !
{
    hello();
    maths();

    #[cfg(test)]
    test_main();

    loop {}
}

fn hello()
{
    println!("Hello, world!");
}

fn maths()
{
    let a = 4;
    let b = 17;
    println!("{} + {} = {}", a, b, a + b);
}


#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> !
{
    println!("{}", info);
    loop {}
}

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Test])
{
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run(); // new
    }
    exit_qemu(QemuExitCode::Success);
}

// our panic handler in test mode
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> !
{
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
}
