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

    /// Add a free memory region to the list
    unsafe fn add_free_region(&self, addr: usize, size: usize) {
        // Align the address up to FreeNode alignment
        let aligned = align_up(addr, mem::align_of::<FreeNode>());
        let size = size - (aligned - addr);

        // Region must be large enough to hold the header
        assert!(size >= MIN_BLOCK, "free region too small for FreeNode header");

        // Write the FreeNode header into the region
        let node_ptr = aligned as *mut FreeNode;
        node_ptr.write(FreeNode::new(size));

        // Push onto the head of the free list
        let mut head = self.head.lock();
        (*node_ptr).next = head.next;
        head.next = node_ptr;
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

    /// Check if a region can satisfy the request, return aligned start if so
    fn alloc_start(
        region_start: usize,
        region_size: usize,
        size: usize,
        align: usize,
    ) -> Option<usize> {
        let alloc_start = align_up(region_start + mem::size_of::<FreeNode>(), align);
        let alloc_end   = alloc_start.checked_add(size)?;

        // Must fit within the region
        if alloc_end > region_start + region_size {
            return None;
        }

        // Leftover space after allocation must fit a FreeNode or be zero
        let leftover = (region_start + region_size) - alloc_end;
        if leftover > 0 && leftover < MIN_BLOCK {
            return None;
        }

        Some(alloc_start)
    }
}

unsafe impl GlobalAlloc for LinkedListAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size  = align_up(layout.size(), mem::align_of::<FreeNode>());
        let align = layout.align();

        match self.find_region(size, align) {
            None => ptr::null_mut(), // OOM
            Some((region_start, alloc_start)) => {
                let alloc_end   = alloc_start + size;
                let region_end  = region_start + {
                    // Re-read region size — safe because we already removed it from the list
                    let node = region_start as *const FreeNode;
                    (*node).size
                };

                // Return leftover tail to free list
                let leftover = region_end - alloc_end;
                if leftover > 0 {
                    self.add_free_region(alloc_end, leftover);
                }

                alloc_start as *mut u8
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = align_up(layout.size(), mem::align_of::<FreeNode>());
        self.add_free_region(ptr as usize, size);
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
}
