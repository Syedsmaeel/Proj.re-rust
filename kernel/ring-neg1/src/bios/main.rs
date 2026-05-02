//! BIOS stage2 entry — runs in 64-bit long mode after stage1 ASM
#![no_std]
#![no_main]

use ring_neg1::bios::{BiosBootFlow, E820Entry};
use ring_neg1::menu::BootMenu;

#[no_mangle]
pub extern "C" fn stage2_main() -> ! {
    // 1. Parse E820 map passed by stage1 (at fixed address 0x7000)
    let e820_entries: &[E820Entry] = &[];  // real: read from 0x7000
    let map = BiosBootFlow::parse_e820(e820_entries);

    // 2. VESA framebuffer (1024x768)
    let fb_info = BiosBootFlow::vesa_framebuffer(0xFD00_0000, 1024, 768);

    // 3. Boot menu
    let menu = BootMenu::default_entries();

    // 4. Build handoff
    let handoff = BiosBootFlow::build_handoff(&map, 0x0010_0000, fb_info);

    // 5. Jump to Timux
    loop {}
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop {} }
