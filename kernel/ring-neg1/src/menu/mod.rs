//! Bash++ TUI boot menu — rendered to framebuffer before kernel handoff
//!
//! Ring -1 renders a full TUI boot menu using pixel-level drawing.
//! No font library needed — uses embedded 8x16 bitmap font.
//! The user selects a boot entry; Ring -1 loads that kernel and jumps.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use crate::display::{Framebuffer, Pixel, Color};

/// A single boot entry
#[derive(Debug, Clone)]
pub struct BootEntry {
    pub label:   String,
    pub kernel:  String,   // path to kernel ELF on disk
    pub cmdline: String,
    pub icon:    &'static str,
    pub default: bool,
}

impl BootEntry {
    pub fn new(label: &str, kernel: &str, cmdline: &str, icon: &'static str) -> Self {
        Self {
            label:   String::from(label),
            kernel:  String::from(kernel),
            cmdline: String::from(cmdline),
            icon,
            default: false,
        }
    }
    pub fn as_default(mut self) -> Self { self.default = true; self }
}

/// Result of the boot menu — which entry was selected
#[derive(Debug, Clone)]
pub struct MenuResult {
    pub entry_idx: usize,
    pub entry:     BootEntry,
    pub timeout:   bool,   // auto-booted after timeout
}

/// The full TUI boot menu
pub struct BootMenu {
    pub entries:     Vec<BootEntry>,
    pub selected:    usize,
    pub timeout_sec: u32,
    pub title:       String,
    pub version:     String,
}

impl BootMenu {
    pub fn new() -> Self {
        Self {
            entries:     Vec::new(),
            selected:    0,
            timeout_sec: 5,
            title:       String::from("Timux Boot"),
            version:     String::from("ring-neg1 v0.1.0"),
        }
    }

    pub fn add_entry(&mut self, entry: BootEntry) {
        if entry.default { self.selected = self.entries.len(); }
        self.entries.push(entry);
    }

    pub fn select_next(&mut self) {
        if !self.entries.is_empty() {
            self.selected = (self.selected + 1) % self.entries.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.entries.is_empty() {
            self.selected = self.selected.checked_sub(1).unwrap_or(self.entries.len() - 1);
        }
    }

    pub fn current(&self) -> Option<&BootEntry> {
        self.entries.get(self.selected)
    }

    /// Render the full boot menu to the framebuffer
    /// Layout:
    ///   ┌────────────────────────────────────────────┐
    ///   │          🦀  Timux Boot  ring-neg1          │  ← header
    ///   │────────────────────────────────────────────│
    ///   │  ▶  Timux (default)          ring-2 shell  │  ← entries
    ///   │     Timux (recovery)                       │
    ///   │     Timux (debug)                          │
    ///   │────────────────────────────────────────────│
    ///   │  ↑↓ navigate   ENTER boot   ESC cancel     │  ← help bar
    ///   │  Auto-boot in 5s...                        │
    ///   └────────────────────────────────────────────┘
    pub fn render(&self, fb: &mut Framebuffer, countdown: u32) {
        let w = fb.width();
        let h = fb.height();

        // Background
        fb.clear(Color::DARK_GRAY);

        // Panel dimensions
        let pw = (w * 60 / 100).max(480);
        let ph = (h * 60 / 100).max(300);
        let px = (w - pw) / 2;
        let py = (h - ph) / 2;

        // Panel background
        fb.fill_rect(px, py, pw, ph, Color::PANEL);
        fb.draw_border(px, py, pw, ph, Color::CYAN);

        // Header bar
        fb.fill_rect(px + 1, py + 1, pw - 2, 32, Color::ACCENT);
        fb.hline(px, py + 33, pw, Color::CYAN);

        // Separator before help bar
        fb.hline(px, py + ph - 48, pw, Color::DARK_CYAN);

        // Help bar background
        fb.fill_rect(px + 1, py + ph - 47, pw - 2, 46, Color::ACCENT);

        // Highlight selected entry
        if !self.entries.is_empty() {
            let entry_h = 36u32;
            let entry_y = py + 38 + self.selected as u32 * entry_h;
            fb.fill_rect(px + 1, entry_y, pw - 2, entry_h - 2, Color::HIGHLIGHT);
            fb.draw_border(px + 2, entry_y, pw - 4, entry_h - 2, Color::CYAN);
        }

        // Timeout bar at very bottom of panel
        if countdown > 0 && self.timeout_sec > 0 {
            let bar_w = (pw - 4) * countdown / self.timeout_sec;
            fb.fill_rect(px + 2, py + ph - 6, bar_w, 4, Color::CYAN);
        }

        // Decorative corner accents
        fb.fill_rect(px,      py,      8, 2, Color::ORANGE);
        fb.fill_rect(px,      py,      2, 8, Color::ORANGE);
        fb.fill_rect(px+pw-8, py,      8, 2, Color::ORANGE);
        fb.fill_rect(px+pw-2, py,      2, 8, Color::ORANGE);
        fb.fill_rect(px,      py+ph-2, 8, 2, Color::ORANGE);
        fb.fill_rect(px,      py+ph-8, 2, 8, Color::ORANGE);
        fb.fill_rect(px+pw-8, py+ph-2, 8, 2, Color::ORANGE);
        fb.fill_rect(px+pw-2, py+ph-8, 2, 8, Color::ORANGE);
    }

    /// Default Timux boot entries
    pub fn default_entries() -> Self {
        let mut menu = BootMenu::new();
        menu.add_entry(BootEntry::new(
            "Timux  (default)",
            "/boot/timux.elf",
            "ring0 loglevel=3",
            "🦀",
        ).as_default());
        menu.add_entry(BootEntry::new(
            "Timux  (recovery)",
            "/boot/timux.elf",
            "ring0 recovery=1 loglevel=5",
            "🔧",
        ));
        menu.add_entry(BootEntry::new(
            "Timux  (debug)",
            "/boot/timux.elf",
            "ring0 debug=1 loglevel=7 serial=1",
            "🐛",
        ));
        menu.add_entry(BootEntry::new(
            "Bash++ shell only",
            "/boot/timux.elf",
            "ring2 bashpp=1 shell_only=1",
            "🐚",
        ));
        menu.add_entry(BootEntry::new(
            "Memory test",
            "/boot/memtest.elf",
            "",
            "🧪",
        ));
        menu
    }
}

impl Default for BootMenu { fn default() -> Self { Self::new() } }
