use sushi::{SushiApp, run_app, eframe::egui};

struct DemoApp {
    counter: i32,
}

impl SushiApp for DemoApp {
    fn name(&self) -> &str { "🍣 sushi demo" }
    
    fn update(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🍣 sushi — Native Sovereignty UI");
            ui.separator();
            
            ui.horizontal(|ui| {
                if ui.button("➖").clicked() { self.counter -= 1; }
                ui.label(format!("Count: {}", self.counter));
                if ui.button("➕").clicked() { self.counter += 1; }
            });
            
            if ui.button("Reset").clicked() { self.counter = 0; }
        });
    }
}

fn main() -> sushi::eframe::Result<()> {
    run_app(DemoApp { counter: 0 })
}
