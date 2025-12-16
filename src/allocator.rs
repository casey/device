use std::alloc::{GlobalAlloc, Layout, System};

use std::sync::atomic::{AtomicUsize, Ordering};

pub(crate) struct Allocator;

#[cfg(feature = "allocator")]
#[global_allocator]
static ALLOCATOR: Allocator = Allocator;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

impl Allocator {
  pub(crate) fn allocated() -> usize {
    ALLOCATED.load(Ordering::Relaxed)
  }
}

unsafe impl GlobalAlloc for Allocator {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    let ptr = unsafe { System.alloc(layout) };
    if !ptr.is_null() {
      ALLOCATED.fetch_add(layout.size(), Ordering::Relaxed);
    }
    ptr
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    unsafe {
      System.dealloc(ptr, layout);
    }
    ALLOCATED.fetch_sub(layout.size(), Ordering::Relaxed);
  }
}
