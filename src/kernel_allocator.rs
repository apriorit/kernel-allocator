extern crate alloc;

use alloc::alloc::{AllocError, Allocator};
use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::{NonNull, null_mut, slice_from_raw_parts_mut},
};
use windows_sys::Wdk::System::SystemServices::{ExAllocatePool2, ExFreePool};

const POOL_FLAG_NON_PAGED: u64 = 64; // Non paged pool NX
const POOL_FLAG_PAGED: u64 = 256; // Paged pool

/// Allocator implementation to use with `#[global_allocator]` to allow use
/// of [`core::alloc`].
///
/// SAFETY
/// This allocator is only safe to use for allocations happening at `IRQL`
/// <= `DISPATCH_LEVEL`
#[derive(Default)]
pub struct KernelAllocator<const FLAGS: u64>;

pub type PagedAlloc = KernelAllocator<POOL_FLAG_PAGED>;
pub type NonPagedAlloc = KernelAllocator<POOL_FLAG_NON_PAGED>;

// Tag: Rust Driver memory tag
const MEMORY_TAG: u32 = u32::from_ne_bytes(*b"RDRV");

// SAFETY:
// This is safe because the allocator can never unwind since it can never panic
// and prevents zero-size allocations by returning a null pointer
unsafe impl<const FLAGS: u64> GlobalAlloc for KernelAllocator<FLAGS> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.size() == 0 {
            return null_mut();
        }

        let ptr =
            // SAFETY:
            // Inherently unsafe as a system call. The caller ensures that the the `FLAGS` parameter
            // is a `POOL_FLAGS` value and doesn't try to allocate paged memory at IRQL > DISPATCH_LEVEL
            unsafe { ExAllocatePool2(FLAGS, layout.size(), MEMORY_TAG) };
        if ptr.is_null() {
            return null_mut();
        }

        ptr.cast()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        // SAFETY:
        // Inherently unsafe as a system call. The caller ensures that the the `ptr` parameter
        // is a valid pointer and doesn't try to free a paged memory at IRQL > APC_LEVEL
        unsafe {
            ExFreePool(ptr.cast());
        }
    }
}

// SAFETY:
// This is safe because the allocator can never unwind since it can never panic
// and prevents zero-size allocations by returning a null pointer
unsafe impl<const FLAGS: u64> Allocator for KernelAllocator<FLAGS> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        // SAFETY:
        // The caller ensures that the `layout` value contains a valid data about memory to be allocated
        let ptr = unsafe { self.alloc(layout) };
        if ptr.is_null() {
            Err(AllocError)
        } else {
            let slice = slice_from_raw_parts_mut(ptr, layout.size());

            // SAFETY:
            // The caller ensures that the `slice` value is a valid data slice and
            // it was checked to be non-zero pointerit was checked to be non-zero pointer
            Ok(unsafe { NonNull::new_unchecked(slice) })
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        // SAFETY:
        // The caller ensures that the `ptr` value is a valid pointer previously allocated
        // with KernelAllocator
        unsafe { self.dealloc(ptr.as_ptr(), layout) };
    }
}
