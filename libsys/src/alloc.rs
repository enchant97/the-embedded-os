use core::alloc::{GlobalAlloc, Layout};

use crate::core::abi;

/// The default System allocator.
///
/// Uses kernel to request dynamic memory.
pub struct System {}

unsafe impl GlobalAlloc for System {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        (abi().malloc)(layout.size())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        (abi().free)(ptr)
    }
}
