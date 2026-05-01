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
                                break;
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
