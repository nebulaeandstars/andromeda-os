#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(andromeda_os::test_runner)]
#![reexport_test_harness_main = "run_test"]

use core::panic::PanicInfo;

use andromeda_os::memory::BootInfoFrameAllocator;
use andromeda_os::vga::Color::*;
use andromeda_os::vga::VGA_WRITER;
use andromeda_os::{halt, memory, print, println, vga};
use bootloader::BootInfo;
use x86_64::structures::paging::{OffsetPageTable, Page, Translate};
use x86_64::VirtAddr;

fn main(
    mut mem_map: OffsetPageTable, mut frame_allocator: BootInfoFrameAllocator,
) {
    vga::with_color(LightCyan, Black, || {
        println!("Hello, world!\n");
    });

    // map an unused page
    let page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
    memory::create_example_mapping(page, &mut mem_map, &mut frame_allocator);

    // write the string `New!` to the screen through the new mapping
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };
}

bootloader::entry_point!(kernel_start);
fn kernel_start(boot_info: &'static BootInfo) -> ! {
    andromeda_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mem_map = unsafe { memory::init(phys_mem_offset) };

    let mut frame_allocator =
        unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    #[cfg(test)]
    run_test();
    #[cfg(not(test))]
    main(mem_map, frame_allocator);

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
