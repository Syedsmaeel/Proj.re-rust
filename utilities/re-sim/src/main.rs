use std::io::{self, Write};
use std::thread;
use std::time::Duration;

fn main() {
    // 1. Stealth Decoy
    println!("GRUB LOADING...");
    thread::sleep(Duration::from_secs(1));
    println!("WELCOME TO GRUB VERSION 2.06");
    println!("Booting in 5 seconds...");
    
    // Wait for secret sequence (for simulation, we use 'unlock')
    print!("> ");
    let _ = io::stdout().flush();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("failed to read stdin");
    
    if input.trim() == "unlock" {
        show_dashboard();
    } else {
        println!("Error: unknown bootloader command.");
    }
}

fn show_dashboard() {
    // Clear screen
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    
    println!("\x1b[35m══════════════════════════════════════════════════════════════════════\x1b[0m");
    println!("\x1b[1m\x1b[36m             TIMUX SOVEREIGN DASHBOARD [SIMULATION]             \x1b[0m");
    println!("\x1b[35m══════════════════════════════════════════════════════════════════════\x1b[0m");
    println!("");
    println!("  [ \x1b[32m BLUEPRINTS\x1b[0m ]              [ \x1b[32m FRACTAL HIERARCHY\x1b[0m ]");
    println!("  ┌──────────────────┐             ┌────────────────────────┐");
    println!("  │ 󰙔 Security-Base  │             │   \x1b[35mRING -1 (SUBSTRATE)\x1b[0m  │");
    println!("  │ \x1b[46m Dev-Fractal   \x1b[0m │ ──────────▶ │   script: \"root.sh\"    │");
    println!("  │ 󰒋 Guest-Silo     │             └───────────┬────────────┘");
    println!("  └──────────────────┘                         │");
    println!("                                  ┌────────────┴────────────┐");
    println!("                                  ▼                         ▼");
    println!("                       ┌────────────────────┐    ┌────────────────────┐");
    println!("                       │ \x1b[33mSUB-KERNEL: NATIVE\x1b[0m │    │ \x1b[33mSUB-KERNEL: GUEST\x1b[0m  │");
    println!("                       │ OS: Timux-Native   │    │ OS: Linux (v6.1)   │");
    println!("                       │ RINGS: 5 (R0-R4)   │    │ RINGS: 2 (R0, R3)  │");
    println!("                       └────────────────────┘    └────────────────────┘");
    println!("");
    println!("\x1b[35m──────────────────────────────────────────────────────────────────────\x1b[0m");
    println!(" [ \x1b[31mSTATUS: FRACTAL-BOOTED\x1b[0m ]  [ \x1b[32mCAPS: 21 ACTIVE\x1b[0m ]  [ \x1b[34mMEM: GATED\x1b[0m ]");
    println!("\x1b[35m──────────────────────────────────────────────────────────────────────\x1b[0m");
}
