#![no_std]
#![no_main]

use timux::boot::{BootInfo, KernelAlloc, KernelState};

#[global_allocator]
static GLOBAL_ALLOC: KernelAlloc = KernelAlloc;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let boot_info = BootInfo::minimal(0x8020_0000);
    let mut kernel = unsafe { KernelState::init(&boot_info) };
    kernel.spawn_init_sk();
    loop {
        unsafe { core::arch::asm!("wfi"); }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        unsafe { core::arch::asm!("wfi"); }
    }
}
