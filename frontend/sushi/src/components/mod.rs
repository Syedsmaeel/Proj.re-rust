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
