#![no_std]
#![no_main]

use core::panic::PanicInfo;

use andromeda_os::{
    exit_qemu, halt, serial_print, serial_println, QemuExitCode,
};

bootloader::entry_point!(test_kernel_start);
fn test_kernel_start(boot_info: &'static bootloader::BootInfo) -> ! {
    should_fail();
    serial_println!("[test did not panic]");
    exit_qemu(QemuExitCode::Failed);
    halt();
}

fn should_fail() {
    serial_print!("should_panic::should_fail...\t");
    assert_eq!(0, 1);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    halt();
}
