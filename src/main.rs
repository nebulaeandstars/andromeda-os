#![no_std]
#![no_main]

mod vga;

use vga::VGA_WRITER;

static HELLO: &[u8] = b"Hello, world!";

#[no_mangle]
pub extern "C" fn _start() -> !
{
    hello();
    maths();
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

use core::panic::PanicInfo;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> !
{
    println!("{}", info);
    loop {}
}
