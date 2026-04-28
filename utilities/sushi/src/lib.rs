pub mod vector;
pub use vector::Canvas;

pub struct Shape {
    pub path: lyon::path::Path,
    pub color: [u8; 4],
}

impl Shape {
    pub fn rect(x: f32, y: f32, w: f32, h: f32, color: [u8; 4]) -> Self {
        use lyon::path::builder::PathBuilder;
        let mut builder = lyon::path::Path::builder();
        builder.add_rectangle(&lyon::math::rect(x, y, w, h), lyon::path::Winding::Positive);
        Self { path: builder.build(), color }
    }
}
