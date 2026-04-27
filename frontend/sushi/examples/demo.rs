use sushi::{SushiApp, run_app, eframe::egui, components::{card, primary_button}};

struct DemoApp {
    counter: i32,
}

impl SushiApp for DemoApp {
    fn name(&self) -> &str { "🍣 sushi - Professional Demo" }
    
    fn update(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🍣 sushi framework");
            ui.add_space(10.0);

            // Using the new Card component
            card(ui, |ui| {
                ui.label("Interactive Counter Card");
                ui.separator();
                
                ui.horizontal(|ui| {
                    if ui.button("➖").clicked() { self.counter -= 1; }
                    ui.label(format!(" Value: {} ", self.counter));
                    if ui.button("➕").clicked() { self.counter += 1; }
                });

                ui.add_space(10.0);
                
                // Using the new Primary Button component
                if primary_button(ui, "Reset Counter").clicked() {
                    self.counter = 0;
                }
            });
        });
    }
}

fn main() -> sushi::eframe::Result<()> {
    run_app(DemoApp { counter: 0 })
}
