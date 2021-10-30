use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PhysFrame, Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};

/// Initialize a new OffsetPageTable.
///
/// # Safety
/// Unsafe because the caller must guarantee that the complete physical memory
/// is mapped to virtual memory at the passed `physical_memory_offset`.
pub unsafe fn init(
    physical_memory_offset: VirtAddr,
) -> OffsetPageTable<'static> {
    let (table_frame, _) = Cr3::read();

    let start_address = table_frame.start_address();
    let virtual_address = physical_memory_offset + start_address.as_u64();
    let level_4_table = &mut *virtual_address.as_mut_ptr();

    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

pub fn create_example_mapping(
    page: Page, mem_map: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result =
        unsafe { mem_map.map_to(page, frame, flags, frame_allocator) };
    map_to_result.expect("map_to failed").flush();
}

pub struct EmptyFrameAllocator;
unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}

/// A FrameAllocator that returns usable frames from the bootloader's memory
/// map.
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next_frame: usize,
}

impl BootInfoFrameAllocator {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// # Safety
    /// Unsafe because the caller must guarantee that the passed memory map is
    /// valid. The main requirement is that all frames that are marked as
    /// `USABLE` in it are really unused.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator { memory_map, next_frame: 0 }
    }

    /// Returns an iterator over the usable frames specified in the memory map.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // get usable regions from memory map
        let usable_regions = self
            .memory_map
            .iter()
            .filter(|r| r.region_type == MemoryRegionType::Usable);

        // map each region to its address range
        let addr_ranges =
            usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());

        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));

        // create `PhysFrame` types from the start addresses
        frame_addresses
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next_frame);
        self.next_frame += 1;
        frame
    }
}
