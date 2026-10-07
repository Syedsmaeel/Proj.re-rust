//! Timux Heap Allocator — Linked List Allocator
//!
//! A classic free-list allocator that runs on bare metal with no OS beneath it.
//!
//! How it works:
//! 1. At boot, the kernel hands us a contiguous block of physical memory
//! 2. We split it into a linked list of free nodes
//! 3. alloc()  — walk the list, find a big enough node, split it, return it
//! 4. dealloc()— merge the freed block back into the list (coalescing)
//!
//! Memory layout of a free node:
//!
//!  ┌──────────────┬──────────────┬────────────────────────┐
//!  │   size: usize│  next: *mut  │   (free space)         │
//!  └──────────────┴──────────────┴────────────────────────┘
//!  ^--- FreeNode header (16 bytes on 64-bit)

use core::alloc::{GlobalAlloc, Layout};
use core::mem;
use core::ptr;
use spin::Mutex;

/// Minimum allocation size — prevents tiny useless nodes
const MIN_BLOCK: usize = mem::size_of::<FreeNode>();

/// A node in the free list
struct FreeNode {
    size: usize,
    next: *mut FreeNode,
}

// SAFETY: FreeNode is only accessed through the Mutex-protected allocator
unsafe impl Send for FreeNode {}

impl FreeNode {
    const fn new(size: usize) -> Self {
        Self { size, next: ptr::null_mut() }
    }

    /// Start address of usable memory (after header)
    fn start_addr(&self) -> usize {
        self as *const _ as usize
    }

    /// End address of this node's region
    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

/// GhostAlloc Trait: Marks memory for volatile, zero-commit storage.
pub trait GhostAlloc {
    fn mark_ghost(&mut self, ptr: *mut u8, size: usize);
    fn purge_ghost(&mut self);
}

impl GhostAlloc for LinkedListAllocator {
    fn mark_ghost(&mut self, _ptr: *mut u8, _size: usize) {
        // Implementation: Mark memory pages as volatile for panic-erasure
    }

    fn purge_ghost(&mut self) {
        // Implementation: Wipe all pages marked as 'ghost'
    }
}

/// The linked-list allocator
pub struct LinkedListAllocator {
    head: Mutex<FreeNode>,
}

impl LinkedListAllocator {
    /// Create an empty allocator — must call init() before use
    pub const fn new() -> Self {
        Self {
            head: Mutex::new(FreeNode::new(0)),
        }
    }

    /// Initialize the allocator with a region of physical memory
    ///
    /// # Safety
    /// - `heap_start` must point to valid, unused physical memory
    /// - `heap_size` bytes starting at `heap_start` must be available
    /// - Must only be called once
    pub unsafe fn init(&self, heap_start: usize, heap_size: usize) {
        self.add_free_region(heap_start, heap_size);
    }

    /// Add a free memory region to the list.
    ///
    /// The list is kept sorted by address so that physically adjacent free
    /// blocks can be merged (coalesced). Without merging, the heap fragments
    /// permanently and eventually fails large allocations.
    unsafe fn add_free_region(&self, addr: usize, size: usize) {
        // Align the address up to FreeNode alignment
        let aligned = align_up(addr, mem::align_of::<FreeNode>());
        let size = size - (aligned - addr);

        // Region must be large enough to hold the header
        assert!(size >= MIN_BLOCK, "free region too small for FreeNode header");

        let node_ptr = aligned as *mut FreeNode;
        node_ptr.write(FreeNode::new(size));

        let mut head = self.head.lock();

        // Find the insertion point: `prev` is the last node whose address is
        // lower than the new region (the list head acts as the sentinel).
        let mut prev: *mut FreeNode = &mut *head;
        while !(*prev).next.is_null() && ((*prev).next as usize) < aligned {
            prev = (*prev).next;
        }

        (*node_ptr).next = (*prev).next;
        (*prev).next = node_ptr;

        // Merge with the following block if they touch.
        let next = (*node_ptr).next;
        if !next.is_null() && (*node_ptr).end_addr() == next as usize {
            (*node_ptr).size += (*next).size;
            (*node_ptr).next = (*next).next;
        }

        // Merge with the preceding block if they touch (never the sentinel,
        // whose size is 0 and which does not live in the heap).
        let head_ptr: *mut FreeNode = &mut *head;
        if prev != head_ptr && (*prev).end_addr() == aligned {
            (*prev).size += (*node_ptr).size;
            (*prev).next = (*node_ptr).next;
        }
    }

    /// Find a free region that fits the layout, remove it from the list
    /// Returns (region_start, alloc_start)
    fn find_region(
        &self,
        size: usize,
        align: usize,
    ) -> Option<(usize, usize)> {
        let mut head = self.head.lock();
        let mut current = &mut *head as *mut FreeNode;

        // Walk the free list
        while !unsafe { (*current).next }.is_null() {
            let region = unsafe { (*current).next };
            let region_start = region as usize;
            let region_size  = unsafe { (*region).size };

            // Find aligned allocation start within this region
            if let Some(alloc_start) = Self::alloc_start(region_start, region_size, size, align) {
                // Remove this node from the list
                unsafe { (*current).next = (*region).next };
                return Some((region_start, alloc_start));
            }

            current = unsafe { (*current).next };
        }

        None
    }

    /// Check if a region can satisfy the request, return aligned start if so.
    ///
    /// Allocations are carved directly out of the free block: no header is
    /// kept in front of the returned pointer (a header would be leaked every
    /// time the block is freed). Any gap before or after the allocation must
    /// be either empty or large enough to become a free block itself.
    fn alloc_start(
        region_start: usize,
        region_size: usize,
        size: usize,
        align: usize,
    ) -> Option<usize> {
        let alloc_start = align_up(region_start, align);
        let alloc_end   = alloc_start.checked_add(size)?;
        let region_end  = region_start + region_size;

        if alloc_end > region_end {
            return None;
        }

        let front = alloc_start - region_start;
        let back  = region_end - alloc_end;
        if (front > 0 && front < MIN_BLOCK) || (back > 0 && back < MIN_BLOCK) {
            return None;
        }

        Some(alloc_start)
    }
}

/// Every block handed out is at least one `FreeNode` big (so it can be put
/// back on the free list) and a multiple of the node alignment.
#[inline]
fn block_size(layout: &Layout) -> usize {
    align_up(
        core::cmp::max(layout.size(), MIN_BLOCK),
        mem::align_of::<FreeNode>(),
    )
}

unsafe impl GlobalAlloc for LinkedListAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size  = block_size(&layout);
        let align = layout.align();

        match self.find_region(size, align) {
            None => ptr::null_mut(), // OOM
            Some((region_start, alloc_start)) => {
                // Region size is re-read from the removed node, which is
                // still intact because nothing has overwritten it yet.
                let region_end = region_start + (*(region_start as *const FreeNode)).size;
                let alloc_end  = alloc_start + size;

                // Return any gap before / after the allocation to the list.
                if alloc_start > region_start {
                    self.add_free_region(region_start, alloc_start - region_start);
                }
                if region_end > alloc_end {
                    self.add_free_region(alloc_end, region_end - alloc_end);
                }

                alloc_start as *mut u8
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.add_free_region(ptr as usize, block_size(&layout));
    }
}

// ─── Alignment helpers ───────────────────────────────────────────────────────

/// Round `addr` up to the nearest multiple of `align`
/// `align` must be a power of two
#[inline]
pub fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

/// Round `addr` down to the nearest multiple of `align`
#[inline]
pub fn align_down(addr: usize, align: usize) -> usize {
    addr & !(align - 1)
}

// ─── Tests (run with std for host-side verification) ─────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align_up() {
        assert_eq!(align_up(0, 8), 0);
        assert_eq!(align_up(1, 8), 8);
        assert_eq!(align_up(8, 8), 8);
        assert_eq!(align_up(9, 8), 16);
        assert_eq!(align_up(1023, 1024), 1024);
    }

    #[test]
    fn test_align_down() {
        assert_eq!(align_down(8, 8), 8);
        assert_eq!(align_down(9, 8), 8);
        assert_eq!(align_down(15, 8), 8);
        assert_eq!(align_down(16, 8), 16);
    }

    #[repr(align(64))]
    struct Heap([u8; 4096]);

    fn fresh(heap: &mut Heap) -> LinkedListAllocator {
        let a = LinkedListAllocator::new();
        unsafe { a.init(heap.0.as_mut_ptr() as usize, heap.0.len()); }
        a
    }

    #[test]
    fn alloc_free_cycles_do_not_erode_heap() {
        let mut heap = Heap([0; 4096]);
        let a = fresh(&mut heap);
        let l = Layout::from_size_align(64, 8).unwrap();
        for _ in 0..1000 {
            let p = unsafe { a.alloc(l) };
            assert!(!p.is_null());
            unsafe { a.dealloc(p, l) };
        }
        // Everything is free again: the whole heap must be one block.
        let big = Layout::from_size_align(4096, 16).unwrap();
        assert!(!unsafe { a.alloc(big) }.is_null());
    }

    #[test]
    fn tiny_allocations_can_be_freed() {
        let mut heap = Heap([0; 4096]);
        let a = fresh(&mut heap);
        for size in [1usize, 8, 15] {
            let l = Layout::from_size_align(size, 1).unwrap();
            let p = unsafe { a.alloc(l) };
            assert!(!p.is_null());
            unsafe { a.dealloc(p, l) }; // must not panic
        }
    }

    #[test]
    fn freed_neighbours_coalesce() {
        let mut heap = Heap([0; 4096]);
        let a = fresh(&mut heap);
        let l = Layout::from_size_align(1024, 8).unwrap();
        let p1 = unsafe { a.alloc(l) };
        let p2 = unsafe { a.alloc(l) };
        let p3 = unsafe { a.alloc(l) };
        assert!(!p1.is_null() && !p2.is_null() && !p3.is_null());
        unsafe {
            a.dealloc(p2, l);
            a.dealloc(p1, l);
            a.dealloc(p3, l);
        }
        let big = Layout::from_size_align(4096, 16).unwrap();
        assert!(!unsafe { a.alloc(big) }.is_null());
    }

    #[test]
    fn respects_alignment_and_does_not_overlap() {
        let mut heap = Heap([0; 4096]);
        let a = fresh(&mut heap);
        let l = Layout::from_size_align(100, 64).unwrap();
        let p1 = unsafe { a.alloc(l) } as usize;
        let p2 = unsafe { a.alloc(l) } as usize;
        assert!(p1 != 0 && p2 != 0);
        assert_eq!(p1 % 64, 0);
        assert_eq!(p2 % 64, 0);
        assert!(p1 + 100 <= p2 || p2 + 100 <= p1, "allocations overlap");
    }

    #[test]
    fn exhaustion_returns_null_instead_of_panicking() {
        let mut heap = Heap([0; 4096]);
        let a = fresh(&mut heap);
        let l = Layout::from_size_align(8192, 8).unwrap();
        assert!(unsafe { a.alloc(l) }.is_null());
    }
}
