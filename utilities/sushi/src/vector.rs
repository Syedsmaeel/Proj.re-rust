use tiny_skia::*;

pub struct Canvas {
    pub pixmap: Pixmap,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Self {
        Self { pixmap: Pixmap::new(width, height).unwrap() }
    }

    pub fn draw(&mut self, shape: &crate::Shape) {
        let mut paint = Paint::default();
        paint.set_color_rgba8(shape.color[0], shape.color[1], shape.color[2], shape.color[3]);
        paint.anti_alias = true;
        self.pixmap.fill_path(&shape.path, &paint, FillRule::Winding, Transform::identity(), None);
    }
}
