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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_creation() {
        let theme = SushiTheme::dark();
        assert_eq!(theme.rounding, 8.0);
        // Verify our signature Sushi Orange
        assert_eq!(theme.primary.r(), 255);
        assert_eq!(theme.primary.g(), 120);
        assert_eq!(theme.primary.b(), 0);
    }
}
