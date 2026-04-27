use eframe::egui;

/// This trait "injects" Sushi components directly into egui::Ui
pub trait SushiUiExt {
    fn sushi_card<R>(&mut self, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> egui::InnerResponse<R>;
    fn sushi_button(&mut self, text: impl Into<egui::WidgetText>) -> egui::Response;
    fn sushi_header(&mut self, text: &str);
}

impl SushiUiExt for egui::Ui {
    fn sushi_card<R>(&mut self, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> egui::InnerResponse<R> {
        egui::Frame::group(self.style())
            .fill(self.visuals().widgets.noninteractive.bg_fill)
            .rounding(8.0)
            .inner_margin(16.0)
            .show(self, add_contents)
    }

    fn sushi_button(&mut self, text: impl Into<egui::WidgetText>) -> egui::Response {
        self.add(egui::Button::new(text).min_size(egui::vec2(100.0, 32.0)))
    }

    fn sushi_header(&mut self, text: &str) {
        self.add_space(8.0);
        self.heading(text);
        self.add_space(4.0);
    }
}

/// A "Glass" Card with a glow-border
pub fn glass_card<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> egui::InnerResponse<R> {
    let mut frame = egui::Frame::group(ui.style())
        .fill(ui.visuals().widgets.noninteractive.bg_fill)
        .rounding(12.0)
        .inner_margin(20.0)
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(255, 255, 255, 20)));
        
    frame.show(ui, |ui| {
        // Add a subtle top-glow
        let rect = ui.max_rect();
        ui.painter().rect_filled(
            egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x, rect.min.y + 2.0)),
            12.0,
            egui::Color32::from_rgba_premultiplied(0, 255, 240, 40)
        );
        add_contents(ui)
    })
}

/// An Animated "Neon" Button
pub fn neon_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let button = egui::Button::new(
        egui::RichText::new(text)
            .color(egui::Color32::WHITE)
            .strong()
    )
    .fill(egui::Color32::TRANSPARENT)
    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 255, 240)))
    .rounding(8.0)
    .min_size(egui::vec2(120.0, 40.0));
    
    let res = ui.add(button);
    
    // Add glow on hover
    if res.hovered() {
        ui.painter().rect_stroke(
            res.rect.expand(2.0),
            8.0,
            egui::Stroke::new(2.0, egui::Color32::from_rgba_premultiplied(0, 255, 240, 100))
        );
    }
    res
}
