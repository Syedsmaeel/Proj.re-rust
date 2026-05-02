#![no_std]
#![no_main]

use timux::boot::{BootInfo, KernelState, ALLOCATOR};

// Register Timux's linked-list allocator as the global heap allocator
#[global_allocator]
static GLOBAL_ALLOC: LinkedListAllocator = LinkedListAllocator::new();
use timux::mm::LinkedListAllocator;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Heap lives at 1MB physical, 4MB size
    let boot_info = BootInfo::minimal(0x10_0000);

    let mut kernel = unsafe { KernelState::init(&boot_info) };
    kernel.spawn_init_sk();

    // Halt
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // TODO: serial print _info, then halt
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}
