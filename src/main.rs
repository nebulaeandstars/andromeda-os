#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod serial;
mod vga;

use core::panic::PanicInfo;

use vga::VGA_WRITER;

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

pub trait Test
{
    fn run(&self) -> ();
}

impl<T> Test for T
where
    T: Fn(),
{
    fn run(&self)
    {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode
{
    Success = 0x10,
    Failed  = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) -> !
{
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }

    loop {}
}
