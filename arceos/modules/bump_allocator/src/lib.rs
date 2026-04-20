#![no_std]

use core::ptr::null;

use allocator::{AllocError, AllocResult, BaseAllocator, ByteAllocator, PageAllocator};

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const SIZE: usize> {
    begin: usize,
    end: usize,
    b_pos: usize,
    p_pos: usize,
    count: usize,
    pcount: usize,
}

impl<const SIZE: usize> EarlyAllocator<SIZE> {
    pub const fn new() -> Self {
        Self {
            begin: 0,
            end: 0,
            b_pos: 0,
            p_pos: 0,
            count: 0,
            pcount: 0,
        }
    }
}

impl<const SIZE: usize> BaseAllocator for EarlyAllocator<SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.begin = start;
        self.b_pos = start;
        self.end = start + size;
        self.p_pos = start + size;
    }

    fn add_memory(&mut self, start: usize, size: usize) -> allocator::AllocResult {
        todo!();
    }
}

impl<const SIZE: usize> ByteAllocator for EarlyAllocator<SIZE> {
    fn alloc(
        &mut self,
        layout: core::alloc::Layout,
    ) -> allocator::AllocResult<core::ptr::NonNull<u8>> {
        let aligned = (self.b_pos + layout.align() - 1) / layout.align() * layout.align();
        let new_pos = aligned + layout.size();
        if new_pos > self.p_pos {
            return AllocResult::Err(AllocError::NoMemory);
        } else {
            self.b_pos = new_pos;
            self.count += 1;
            return AllocResult::Ok(core::ptr::NonNull::new(aligned as *mut u8).unwrap());
        }
    }

    fn dealloc(&mut self, pos: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        if self.count == 0 {
            self.b_pos = self.begin;
        } else {
            self.count -= 1;
        }
        if self.count == 0 {
            self.b_pos = self.begin;
        }
    }

    fn total_bytes(&self) -> usize {
        todo!()
    }

    fn used_bytes(&self) -> usize {
        self.b_pos - self.begin
    }

    fn available_bytes(&self) -> usize {
        self.p_pos - self.b_pos
    }
}

impl<const SIZE: usize> PageAllocator for EarlyAllocator<SIZE> {
    const PAGE_SIZE: usize = SIZE;

    fn alloc_pages(
        &mut self,
        num_pages: usize,
        align_pow2: usize,
    ) -> allocator::AllocResult<usize> {
        let aligned = self.p_pos / align_pow2 * align_pow2;
        let new_pos = aligned - (num_pages * SIZE);
        if new_pos < self.b_pos {
            return Err(AllocError::NoMemory);
        } else {
            self.pcount += 1;
            self.p_pos = new_pos;
            return Ok(new_pos);
        }
    }

    fn dealloc_pages(&mut self, pos: usize, num_pages: usize) {
        self.pcount -= 1;
    }

    fn total_pages(&self) -> usize {
        todo!()
    }

    fn used_pages(&self) -> usize {
        (self.end - self.p_pos) / SIZE
    }

    fn available_pages(&self) -> usize {
        (self.p_pos - self.b_pos) / SIZE
    }
}
