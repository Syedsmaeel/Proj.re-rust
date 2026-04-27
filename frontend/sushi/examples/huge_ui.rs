use sushi::{SushiApp, run_app, eframe::egui, components::{glass_card, neon_button}, SushiTheme};

struct HugeApp;
impl SushiApp for HugeApp {
    fn name(&self) -> &str { "🍣 sushi - Cyber-Glass" }
    fn theme(&self) -> SushiTheme { SushiTheme::cyber_glass() }
    
    fn update(&mut self, ctx: &egui::Context) {
        // Background Gradient Painting
        let rect = ctx.screen_rect();
        let painter = ctx.layer_painter(egui::LayerId::background());
        painter.rect_filled(
            rect,
            0.0,
            egui::Color32::from_rgb(10, 10, 15)
        );
        // Subtle glow in corner
        painter.circle_filled(egui::pos2(0.0, 0.0), 400.0, egui::Color32::from_rgba_premultiplied(0, 255, 240, 15));

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
            ui.add_space(50.0);
            ui.vertical_centered(|ui| {
                ui.heading(egui::RichText::new("SYSTEM OVERRIDE").size(40.0).strong().color(egui::Color32::WHITE));
                ui.label(egui::RichText::new("Sovereignty Stack v1.0").color(egui::Color32::GRAY));
                
                ui.add_space(40.0);
                
                glass_card(ui, |ui| {
                    ui.set_min_width(400.0);
                    ui.label("CORE STATUS: SECURE");
                    ui.add_space(10.0);
                    
                    ui.horizontal(|ui| {
                        neon_button(ui, "INITIALIZE");
                        neon_button(ui, "REFRACTOR");
                    });
                });
            });
        });
    }
}

fn main() -> sushi::eframe::Result<()> {
    run_app(HugeApp)
}
