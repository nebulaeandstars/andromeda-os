/// An array of available block sizes.
///
/// The sizes must each be power of 2 because they are also used as
/// the block alignment (alignments must be always powers of 2).
const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

/// An optional static reference to a Node.
type BlockHead = Option<&'static mut Node>;

struct Node {
    next: BlockHead,
}

pub struct PoolAllocator {
    list_heads:         [BlockHead; BLOCK_SIZES.len()],
    fallback_allocator: linked_list_allocator::Heap,
}
