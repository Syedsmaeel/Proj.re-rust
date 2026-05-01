#![no_std]
#![no_main]

use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::proto::console::text::Input;
use log::info;
use tbm::graphics::{Renderer, SovereignDashboard, Color};

#[entry]
fn main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).expect("failed to initialize uefi services");
    
    info!("🦀 Timux Boot Manager (TBM) starting...");

    let bt = system_table.boot_services();
    
    // Initialize Graphics
    let gop_handle = bt.get_handle_for_protocol::<GraphicsOutput>()
        .expect("failed to get GOP handle");
    let gop = bt.open_protocol_exclusive::<GraphicsOutput>(gop_handle)
        .expect("failed to open GOP protocol");
    
    let mut gop = gop;
    let renderer = Renderer::new(&mut gop);
    let mut dashboard = SovereignDashboard::new(renderer);

    // Start in Stealth Mode (Decoy Menu)
    dashboard.render();

    // Initialize Input
    let stdin_handle = bt.get_handle_for_protocol::<Input>()
        .expect("failed to get stdin handle");
    let mut stdin = bt.open_protocol_exclusive::<Input>(stdin_handle)
        .expect("failed to open stdin protocol");

    // Phase 3: Sovereign Shield
    if !tbm::crypto::SovereignShield::unlock_kernel(&mut stdin) {
        return Status::ABORTED;
    }
    tbm::crypto::SovereignShield::verify_integrity(&[]);

    let mut secret_sequence = [0u8; 5];
    let mut seq_idx = 0;
    let target_sequence = b"TIMUX";

    // Event Loop
    loop {
        if let Ok(Some(key)) = stdin.read_key() {
            match key {
                uefi::proto::console::text::Key::Printable(c) => {
                    let ch = c.as_char() as u8;
                    
                    if dashboard.is_stealth_active {
                        // Secret sequence logic
                        if ch == target_sequence[seq_idx] {
                            secret_sequence[seq_idx] = ch;
                            seq_idx += 1;
                            if seq_idx == target_sequence.len() {
                                dashboard.is_stealth_active = false;
                                info!("Sovereignty unlocked.");
                            }
                        } else {
                            seq_idx = 0; // Reset on wrong key
                        }
                    } else {
                        // Sovereign Dashboard logic
                        match ch {
                            b'j' | b's' => { // Down
                                dashboard.selected_index = (dashboard.selected_index + 1) % dashboard.blueprints.len();
                            }
                            b'k' | b'w' => { // Up
                                if dashboard.selected_index == 0 {
                                    dashboard.selected_index = dashboard.blueprints.len() - 1;
                                } else {
                                    dashboard.selected_index -= 1;
                                }
                            }
                            b'b' => { // Boot
                                info!("Booting: {}", dashboard.blueprints[dashboard.selected_index].name);
                                
                                // Phase 4: Finalize Kernel Handoff
                                // This is a placeholder for the kernel image data
                                let kernel_elf = include_bytes!("../../../target/x86_64-unknown-none/debug/timux-x86_64");
                                let mut boot_info = BootInfo {
                                    magic: BootInfo::MAGIC,
                                    version: 1,
                                    heap_start: 0x10_0000,
                                    heap_size: 0x40_0000,
                                    framebuffer: renderer.fb_info(),
                                    mmap_addr: 0,
                                    mmap_len: 0,
                                    blueprint_addr: 0,
                                    blueprint_len: 0,
                                    entropy_seed: [0; 32],
                                };

                                let entry = tbm::loader::KernelLoader::load(kernel_elf, &mut boot_info)
                                    .expect("failed to load kernel");

                                info!("Jumping to kernel at {:#x}", entry);
                                
                                // Jump to kernel entry point
                                type KernelEntry = unsafe extern "C" fn(*const BootInfo) -> !;
                                let kernel_main: KernelEntry = unsafe { core::mem::transmute(entry as *const ()) };
                                unsafe { kernel_main(&boot_info); }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
            // Re-render
            dashboard.render();
        }
    }

    info!("Handoff to Timux Kernel (-1)...");
    Status::SUCCESS
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
