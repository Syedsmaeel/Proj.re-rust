use sushi::{SushiApp, run_app, eframe::egui, SushiUiExt, view};

struct MyApp {
    count: i32,
}

impl SushiApp for MyApp {
    fn name(&self) -> &str { "🍣 sushi - Super Easy Mode" }
    
    fn update(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // THE NEW EASY MODE:
            // 1. Direct header call
            ui.sushi_header("Sovereignty Dashboard");

            // 2. Declarative view macro
            view!(ui, {
                // 3. Components called directly on 'ui'
                ui.sushi_card(|ui| {
                    ui.label(format!("Live Counter: {}", self.count));
                    
                    ui.add_space(10.0);
                    
                    ui.horizontal(|ui| {
                        if ui.sushi_button("Increment").clicked() { self.count += 1; }
                        if ui.sushi_button("Reset").clicked() { self.count = 0; }
                    });
                });
            });
        });
    }
}

fn main() -> sushi::eframe::Result<()> {
    run_app(MyApp { count: 0 })
}
