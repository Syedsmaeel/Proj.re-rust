#![no_std]
#![no_main]

use timux::boot::{BootInfo, KernelState, ALLOCATOR};
use timux::mm::LinkedListAllocator;

#[global_allocator]
static GLOBAL_ALLOC: &LinkedListAllocator = &ALLOCATOR;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let boot_info = BootInfo::minimal(0x4000_0000); // typical ARM64 DRAM start
    let mut kernel = unsafe { KernelState::init(&boot_info) };
    kernel.spawn_init_sk();
    loop {
        unsafe { core::arch::asm!("wfe"); }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        unsafe { core::arch::asm!("wfe"); }
    }
}
