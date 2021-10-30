use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr;

use super::Locked;

pub struct BumpAllocator {
    heap_start:  usize,
    heap_end:    usize,
    next:        usize,
    allocations: usize,
}

impl BumpAllocator {
    /// Creates a new empty bump allocator.
    pub const fn new() -> Self {
        BumpAllocator {
            heap_start:  0,
            heap_end:    0,
            next:        0,
            allocations: 0,
        }
    }

    /// Initializes the bump allocator with the given heap bounds.
    ///
    /// This method is unsafe because the caller must ensure that the given
    /// memory range is unused. Also, this method must be called only once.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut bump = self.lock();

        // Align the start address to match the given Layout.
        let alloc_start = super::align_next_unsafe(bump.next, layout.align());

        // Find the end of the memory region, checking for any overflows.
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        // Make sure that the allocator's bounds are being respected.
        if alloc_end > bump.heap_end {
            return ptr::null_mut();
        }

        // Finally, set up for the next allocation and return the now-aligned
        // start address as a raw pointer.
        bump.next = alloc_end;
        bump.allocations += 1;
        alloc_start as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let mut bump = self.lock();

        bump.allocations -= 1;
        if bump.allocations == 0 {
            bump.next = bump.heap_start;
        }
    }
}
