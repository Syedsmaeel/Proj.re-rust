use tiny_skia::*;

pub struct Shape {
    pub path: Path,
    pub color: [u8; 4],
}

impl Shape {
    pub fn rect(x: f32, y: f32, w: f32, h: f32, color: [u8; 4]) -> Self {
        let path = PathBuilder::from_rect(Rect::from_xywh(x, y, w, h).unwrap());
        Self { path, color }
    }
}

pub mod vector;
pub use vector::Canvas;
