#![no_std]
#![cfg_attr(test, no_main)]
#![feature(const_mut_refs)]
#![feature(custom_test_frameworks)]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "run_test"]

extern crate alloc;

pub mod allocator;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod serial;
pub mod vga;

use core::panic::PanicInfo;

use x86_64::structures::paging::OffsetPageTable;

pub fn init(
    boot_info: &'static bootloader::BootInfo,
) -> (OffsetPageTable, memory::BootInfoFrameAllocator) {
    use x86_64::VirtAddr;

    gdt::init();

    // Load the Interrupt Descriptor Table.
    interrupts::init_idt();

    // Enable the 8259 PICs
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mem_map = unsafe { memory::init(phys_mem_offset) };

    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mem_map, &mut frame_allocator)
        .expect("heap initialization failed");

    (mem_map, frame_allocator)
}

/// Enter a low-power infinite loop.
pub fn halt() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

/// Entry point for tests
#[cfg(test)]
bootloader::entry_point!(test_kernel_start);

#[cfg(test)]
fn test_kernel_start(boot_info: &'static bootloader::BootInfo) -> ! {
    init(&boot_info);
    run_test();
    halt()
}

pub trait Test {
    fn run(&self) -> ();
}

impl<T> Test for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Test]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed  = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }

    halt()
}
