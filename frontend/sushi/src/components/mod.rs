use eframe::egui;

/// A card-like container for grouping UI elements
pub fn card<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> egui::InnerResponse<R> {
    egui::Frame::group(ui.style())
        .fill(ui.visuals().widgets.noninteractive.bg_fill)
        .rounding(8.0)
        .inner_margin(12.0)
        .show(ui, add_contents)
}

/// A styled primary button
pub fn primary_button(ui: &mut egui::Ui, text: impl Into<egui::WidgetText>) -> egui::Response {
    let text = text.into();
    ui.add(egui::Button::new(text).min_size(egui::vec2(80.0, 30.0)))
}
