use eframe::egui;

pub struct SushiTheme {
    pub primary: egui::Color32,
    pub accent: egui::Color32,
    pub glass: egui::Color32,
    pub text: egui::Color32,
}

impl SushiTheme {
    pub fn cyber_glass() -> Self {
        Self {
            primary: egui::Color32::from_rgb(0, 255, 240),   // Neon Cyan
            accent: egui::Color32::from_rgb(255, 0, 255),    // Cyber Pink
            glass: egui::Color32::from_rgba_premultiplied(20, 20, 25, 180), // Tinted Glass
            text: egui::Color32::WHITE,
        }
    }

    pub fn apply(&self, ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();
        visuals.widgets.noninteractive.bg_fill = self.glass;
        visuals.widgets.noninteractive.rounding = 12.0.into();
        visuals.widgets.inactive.rounding = 8.0.into();
        visuals.selection.bg_fill = self.primary.linear_multiply(0.3);
        ctx.set_visuals(visuals);
    }
}
