//! UEFI EFI application entry point — BOOTX64.EFI
#![no_std]
#![no_main]

use ring_neg1::uefi::UefiBootFlow;
use ring_neg1::menu::BootMenu;
use ring_neg1::proto::HANDOFF_MAGIC;

#[no_mangle]
pub extern "efiapi" fn efi_main(
    _image_handle: *mut core::ffi::c_void,
    _system_table: *mut core::ffi::c_void,
) -> usize {
    // 1. Get memory map
    let map = UefiBootFlow::get_memory_map();

    // 2. Init framebuffer (GOP)
    let mut _fb = UefiBootFlow::init_framebuffer();

    // 3. Build boot menu
    let mut menu = BootMenu::default_entries();

    // 4. Render menu to framebuffer (if available)
    // if let Some(ref mut fb) = _fb { menu.render(fb, 5); }

    // 5. Auto-select default entry (timeout)
    let entry = menu.current().expect("no boot entries");

    // 6. Build handoff
    let handoff = UefiBootFlow::build_handoff(&map, 0x0050_0000_u64);

    // 7. ExitBootServices (real UEFI only)
    // exit_boot_services(image_handle, map_key);

    // 8. Jump to Timux
    // SAFETY: on real hardware this jumps to the kernel entry
    // let entry_fn: extern "C" fn(*const HandoffInfo) -> ! = unsafe { core::mem::transmute(handoff.kernel_entry) };
    // entry_fn(&handoff as *const _);

    loop {}
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop {} }
