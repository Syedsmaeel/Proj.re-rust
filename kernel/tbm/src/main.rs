#![no_std]
#![no_main]

use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::proto::console::text::Input;
use log::info;
use tbm::graphics::{Renderer, SovereignDashboard};
use tbm::protocol::BootInfo;

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

    // Event Loop
    loop {
        if let Ok(Some(key)) = stdin.read_key() {
            match key {
                uefi::proto::console::text::Key::Special(s) => {
                    if s == uefi::proto::console::text::ScanCode::F12 {
                        if dashboard.is_stealth_active {
                            dashboard.is_stealth_active = false;
                            info!("Sovereignty unlocked via F12.");
                        }
                    }
                }
                uefi::proto::console::text::Key::Printable(c) => {
                    let ch = u16::from(c) as u8;

                    if !dashboard.is_stealth_active {
                        match ch {
                            b'j' | b's' => {
                                dashboard.selected_index = (dashboard.selected_index + 1) % dashboard.blueprints.len();
                            }
                            b'k' | b'w' => {
                                if dashboard.selected_index == 0 {
                                    dashboard.selected_index = dashboard.blueprints.len() - 1;
                                } else {
                                    dashboard.selected_index -= 1;
                                }
                            }
                            b'b' => {
                                info!("Booting: {}", dashboard.blueprints[dashboard.selected_index].name);
                                info!("Kernel loading skipped (missing binary)");
                                break;
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
            dashboard.render();
        }
    }

    Status::SUCCESS
}
