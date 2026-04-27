use eframe::egui;

pub struct SushiTheme {
    pub primary: egui::Color32,
    pub background: egui::Color32,
    pub rounding: f32,
}

impl SushiTheme {
    pub fn dark() -> Self {
        Self {
            primary: egui::Color32::from_rgb(255, 120, 0), // Sushi Orange
            background: egui::Color32::from_rgb(18, 18, 18),
            rounding: 8.0,
        }
    }

    pub fn apply(&self, ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();
        visuals.widgets.active.bg_fill = self.primary;
        visuals.widgets.hovered.bg_fill = self.primary.linear_multiply(0.8);
        visuals.window_rounding = self.rounding.into();
        ctx.set_visuals(visuals);
    }
}
