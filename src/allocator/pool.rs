use alloc::alloc::{GlobalAlloc, Layout};
use core::{mem, ptr};

use super::Locked;

/// An array of available block sizes.
///
/// The sizes must each be power of 2 because they are also used as
/// the block alignment (alignments must be always powers of 2).
const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

/// Determines which block list to use for a given layout.
///
/// Returns an index into the `BLOCK_SIZES` array if the requested size fits
/// into one of the sizes listed. Returns None if the size is too large.
fn list_index(layout: &Layout) -> Option<usize> {
    let required_block_size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
}

/// An optional static reference to a Node.
type BlockHead = Option<&'static mut Node>;

struct Node {
    next: BlockHead,
}

const NODE_SIZE: usize = mem::size_of::<Node>();
const NODE_ALIGN: usize = mem::align_of::<Node>();

pub struct PoolAllocator {
    list_heads:         [BlockHead; BLOCK_SIZES.len()],
    fallback_allocator: linked_list_allocator::Heap,
}

impl PoolAllocator {
    /// Creates a new PoolAllocator. Will need to be initialised.
    pub const fn new() -> Self {
        const EMPTY: Option<&'static mut Node> = None;
        PoolAllocator {
            list_heads:         [EMPTY; BLOCK_SIZES.len()],
            fallback_allocator: linked_list_allocator::Heap::empty(),
        }
    }

    /// Initialize the allocator with the given heap bounds.
    ///
    /// Unsafe because the caller must guarantee that the given heap bounds are
    /// valid and that the heap is unused. This method must be called only once.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.fallback_allocator.init(heap_start, heap_size);

        // only the fallback allocator was initialised (rather than any of the
        // list heads), as they'll be lazily initialised later.
    }

    /// Allocates using the fallback allocator.
    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        match self.fallback_allocator.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => ptr::null_mut(),
        }
    }
}

unsafe impl GlobalAlloc for Locked<PoolAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();

        // Peek into the corresponding block list for the layout.
        match list_index(&layout) {
            Some(index) => match allocator.list_heads[index].take() {
                // If there is a node at the list head, move it to the *next*
                // node and return a pointer to the old (now empty) head.
                Some(node) => {
                    allocator.list_heads[index] = node.next.take();
                    (node as *mut Node) as *mut u8
                },
                // Otherwise, the list must be empty. Allocate a block using the
                // fallback allocator to start a new list.
                None => {
                    let block_size = BLOCK_SIZES[index];
                    let block_align = block_size;

                    let layout =
                        Layout::from_size_align(block_size, block_align)
                            .unwrap();
                    allocator.fallback_alloc(layout)
                },
            },
            // If the layout doesn't fit anywhere, use the fallback allocator.
            None => allocator.fallback_alloc(layout),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();

        // Peek into the corresponding block list for the layout.
        match list_index(&layout) {
            // If the block fits into a list:
            Some(index) => {
                assert!(NODE_SIZE <= BLOCK_SIZES[index]);
                assert!(NODE_ALIGN <= BLOCK_SIZES[index]);

                // Take whatever is currently at the list head,
                let new_node =
                    Node { next: allocator.list_heads[index].take() };

                // use it to overwrite the deallocated node,
                let ptr = ptr as *mut Node;
                ptr.write(new_node);

                // and set the list head to the given address.
                allocator.list_heads[index] = Some(&mut *ptr);
            },
            // If the layout doesn't fit anywhere, it must have been allocated
            // using the fallback allocator, so we'll use that here as well.
            None => {
                let ptr = ptr::NonNull::new(ptr).unwrap();
                allocator.fallback_allocator.deallocate(ptr, layout);
            },
        }
    }
}
