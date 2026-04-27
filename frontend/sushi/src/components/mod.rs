use eframe::egui;

/// The core trait for building reusable UI components in Sushi
pub trait Component {
    /// Render the component into the provided UI context
    fn show(&mut self, ui: &mut egui::Ui);
}

/// Helper function to render any component easily
pub fn render<C: Component>(ui: &mut egui::Ui, mut component: C) {
    component.show(ui);
}

// -- Built-in Components --

pub fn card<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> egui::InnerResponse<R> {
    egui::Frame::group(ui.style())
        .fill(ui.visuals().widgets.noninteractive.bg_fill)
        .rounding(8.0)
        .inner_margin(12.0)
        .show(ui, add_contents)
}

pub fn primary_button(ui: &mut egui::Ui, text: impl Into<egui::WidgetText>) -> egui::Response {
    ui.add(egui::Button::new(text).min_size(egui::vec2(80.0, 30.0)))
}
