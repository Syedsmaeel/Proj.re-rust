use uefi::prelude::*;
use uefi::proto::console::gop::{GraphicsOutput, PixelFormat};
use re_core::protocol::FramebufferInfo;

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255 };
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
    pub const DRACULA_BG: Color = Color { r: 40, g: 42, b: 54 };
    pub const DRACULA_FG: Color = Color { r: 248, g: 248, b: 242 };
    pub const DRACULA_CYAN: Color = Color { r: 139, g: 233, b: 253 };
    pub const DRACULA_PURPLE: Color = Color { r: 189, g: 147, b: 249 };
}

pub struct Renderer<'a> {
    gop: &'a mut GraphicsOutput<'a>,
    fb_info: FramebufferInfo,
}

impl<'a> Renderer<'a> {
    pub fn new(gop: &'a mut GraphicsOutput<'a>) -> Self {
        let mode = gop.current_mode_info();
        let (width, height) = mode.resolution();
        let fb_info = FramebufferInfo {
            addr: gop.frame_buffer().as_mut_ptr() as u64,
            size: gop.frame_buffer().size(),
            width: width as u32,
            height: height as u32,
            pitch: width as u32, // Simplified for now
        };

        Self { gop, fb_info }
    }

    pub fn clear(&mut self, color: Color) {
        let (width, height) = self.gop.current_mode_info().resolution();
        self.draw_rect(0, 0, width as u32, height as u32, color);
    }

    pub fn draw_rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: Color) {
        let mode = self.gop.current_mode_info();
        let (res_x, res_y) = mode.resolution();
        let mut fb = self.gop.frame_buffer();

        for i in y..core::cmp::min(y + height, res_y as u32) {
            for j in x..core::cmp::min(x + width, res_x as u32) {
                let pixel_index = (i as usize * res_x as usize) + j as usize;
                let pixel_offset = pixel_index * 4;
                
                // Assuming BGR format for most UEFI systems, but should check PixelFormat
                fb.as_mut_ptr()[pixel_offset]     = color.b;
                fb.as_mut_ptr()[pixel_offset + 1] = color.g;
                fb.as_mut_ptr()[pixel_offset + 2] = color.r;
                fb.as_mut_ptr()[pixel_offset + 3] = 255; // Alpha
            }
        }
    }

    pub fn draw_border(&mut self, x: u32, y: u32, width: u32, height: u32, thickness: u32, color: Color) {
        // Top
        self.draw_rect(x, y, width, thickness, color);
        // Bottom
        self.draw_rect(x, y + height - thickness, width, thickness, color);
        // Left
        self.draw_rect(x, y, thickness, height, color);
        // Right
        self.draw_rect(x + width - thickness, y, thickness, height, color);
    }

    pub fn fb_info(&self) -> FramebufferInfo {
        self.fb_info
    }
}

pub const FONT_8X8: [[u8; 8]; 14] = [
    [0x3C, 0x42, 0x42, 0x7E, 0x42, 0x42, 0x42, 0x00], // A
    [0x7C, 0x42, 0x42, 0x7C, 0x42, 0x42, 0x7C, 0x00], // B
    [0x3C, 0x42, 0x40, 0x40, 0x40, 0x42, 0x3C, 0x00], // C
    [0x78, 0x44, 0x42, 0x42, 0x42, 0x44, 0x78, 0x00], // D
    [0x7E, 0x40, 0x40, 0x78, 0x40, 0x40, 0x7E, 0x00], // E
    [0x7E, 0x40, 0x40, 0x78, 0x40, 0x40, 0x40, 0x00], // F
    [0x3C, 0x42, 0x40, 0x4E, 0x42, 0x42, 0x3C, 0x00], // G
    [0x42, 0x42, 0x42, 0x7E, 0x42, 0x42, 0x42, 0x00], // H
    [0x3C, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00], // I
    [0x1E, 0x04, 0x04, 0x04, 0x04, 0x44, 0x38, 0x00], // J
    [0x44, 0x48, 0x50, 0x60, 0x50, 0x48, 0x44, 0x00], // K
    [0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x7E, 0x00], // L
    [0x42, 0x66, 0x5A, 0x42, 0x42, 0x42, 0x42, 0x00], // M
    [0x7E, 0x42, 0x42, 0x42, 0x42, 0x42, 0x7E, 0x00], // Default/Box
];

impl<'a> Renderer<'a> {
    pub fn draw_char(&mut self, x: u32, y: u32, c: char, color: Color) {
        let idx = match c.to_ascii_uppercase() {
            'A' => 0, 'B' => 1, 'C' => 2, 'D' => 3, 'E' => 4,
            'F' => 5, 'G' => 6, 'H' => 7, 'I' => 8, 'J' => 9,
            'K' => 10, 'L' => 11, 'M' => 12,
            _ => 13,
        };
        let glyph = FONT_8X8[idx];

        for (row, bits) in glyph.iter().enumerate() {
            for col in 0..8 {
                if (bits >> (7 - col)) & 1 == 1 {
                    self.draw_rect(x + col as u32, y + row as u32, 1, 1, color);
                }
            }
        }
    }

    pub fn draw_text(&mut self, x: u32, y: u32, text: &str, color: Color) {
        let mut curr_x = x;
        for c in text.chars() {
            if c != ' ' {
                self.draw_char(curr_x, y, c, color);
            }
            curr_x += 10;
        }
    }
}

pub struct Blueprint {
    pub name: &'static str,
    pub description: &'static str,
}

#[derive(Clone, Copy)]
pub enum OsType {
    Native,
    Linux,
    NestedBase,
}

pub struct SubKernelNode {
    pub name: &'static str,
    pub os: OsType,
    pub rings: u8,
}

pub struct SovereignDashboard<'a> {
    renderer: Renderer<'a>,
    pub blueprints: [Blueprint; 3],
    pub selected_index: usize,
    pub is_stealth_active: bool,
    pub nodes: [Option<SubKernelNode>; 4],
}

impl<'a> SovereignDashboard<'a> {
    pub fn new(renderer: Renderer<'a>) -> Self {
        let blueprints = [
            Blueprint { name: "DEBIAN-12.ISO", description: "Linux Guest (2-Ring Adaptive)" },
            Blueprint { name: "DEV-FRACTAL", description: "Recursive Software Base" },
            Blueprint { name: "RECOVERY.IMG", description: "Emergency Flat-Ring Tool" },
        ];

        
        let mut nodes: [Option<SubKernelNode>; 4] = [None, None, None, None];
        nodes[0] = Some(SubKernelNode { name: "ROOT", os: OsType::NestedBase, rings: 8 });
        nodes[1] = Some(SubKernelNode { name: "NET", os: OsType::Native, rings: 5 });
        nodes[2] = Some(SubKernelNode { name: "GUEST", os: OsType::Linux, rings: 2 });

        Self { 
            renderer, 
            blueprints, 
            selected_index: 0,
            is_stealth_active: true,
            nodes,
        }
    }

    pub fn render(&mut self) {
        if self.is_stealth_active {
            self.render_decoy();
        } else {
            self.render_layout();
        }
    }

    fn render_decoy(&mut self) {
        let (width, height) = (self.renderer.fb_info.width, self.renderer.fb_info.height);
        self.renderer.clear(Color::BLACK);
        self.renderer.draw_text(width / 2 - 100, height / 2, "GRUB LOADING...", Color::WHITE);
        self.renderer.draw_text(width / 2 - 100, height / 2 + 20, "WELCOME TO GRUB!", Color::WHITE);
    }

    pub fn render_layout(&mut self) {
        let (width, height) = (self.renderer.fb_info.width, self.renderer.fb_info.height);
        
        // Background
        self.renderer.clear(Color::DRACULA_BG);

        // Header
        self.renderer.draw_rect(0, 0, width, 40, Color::DRACULA_PURPLE);
        self.renderer.draw_text(20, 15, "TIMUX SOVEREIGN DASHBOARD", Color::WHITE);
        
        // Left Pane: Blueprint Gallery & Ingestion
        let left_width = (width as f32 * 0.3) as u32;
        self.renderer.draw_border(10, 50, left_width, height - 100, 2, Color::DRACULA_CYAN);
        self.renderer.draw_text(20, 60, "BLUEPRINTS & ASSETS", Color::DRACULA_CYAN);

        for (i, bp) in self.blueprints.iter().enumerate() {
            let color = if i == self.selected_index { Color::DRACULA_PURPLE } else { Color::WHITE };
            let y_off = 90 + (i as u32 * 30);
            if i == self.selected_index {
                self.renderer.draw_rect(15, y_off - 5, left_width - 10, 25, Color::DRACULA_CYAN);
            }
            let icon = if bp.name.contains(".ISO") || bp.name.contains(".IMG") { "󰒋 " } else { " " };
            self.renderer.draw_text(20, y_off, bp.name, color);
        }
        
        // Right Pane: Fractal Hierarchy Map
        let right_x = left_width + 20;
        let right_width = width - right_x - 10;
        self.renderer.draw_border(right_x, 50, right_width, height - 100, 2, Color::DRACULA_CYAN);
        self.renderer.draw_text(right_x + 10, 60, "FRACTAL HIERARCHY", Color::DRACULA_CYAN);

        self.draw_fractal_map(right_x + 50, 100);

        // Bottom Bar
        self.renderer.draw_rect(0, height - 40, width, 40, Color::DRACULA_PURPLE);
        let status = self.blueprints[self.selected_index].description;
        self.renderer.draw_text(20, height - 25, status, Color::WHITE);
    }

    fn draw_fractal_map(&mut self, x: u32, y: u32) {
        // Deep Roots (Ring -5 to -1)
        let root_x = x;
        let mut curr_y = y;

        // Draw Deep Sovereignty Stack
        let rings = [
            ("RING -5", Color::DRACULA_PURPLE),
            ("RING -4", Color::DRACULA_PURPLE),
            ("RING -3", Color::DRACULA_PURPLE),
            ("RING -2", Color::DRACULA_PURPLE),
            ("RING -1", Color::DRACULA_CYAN),
        ];

        for (name, color) in rings.iter() {
            self.renderer.draw_rect(root_x, curr_y, 120, 20, *color);
            self.renderer.draw_text(root_x + 5, curr_y + 5, name, Color::WHITE);
            self.renderer.draw_rect(root_x + 60, curr_y + 20, 2, 5, Color::DRACULA_FG);
            curr_y += 25;
        }

        // Sub-Kernels (Children of Ring -1)
        let sub_y = curr_y + 20;
        self.renderer.draw_rect(root_x - 50, sub_y, 100, 30, Color::DRACULA_CYAN);
        self.renderer.draw_text(root_x - 45, sub_y + 10, "SK NATIVE", Color::WHITE);

        self.renderer.draw_rect(root_x + 70, sub_y, 100, 30, Color::DRACULA_CYAN);
        self.renderer.draw_text(root_x + 75, sub_y + 10, "SK GUEST", Color::WHITE);
        
        // Connectors
        self.renderer.draw_rect(root_x + 60, curr_y, 2, 20, Color::DRACULA_FG);
        self.renderer.draw_rect(root_x - 0, sub_y - 20, 120, 2, Color::DRACULA_FG);
    }
}
