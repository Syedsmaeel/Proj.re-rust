use sushi::{SushiApp, run_app, eframe::egui, components::{Component, render}, view};

// 1. Defining a component is just a struct and a trait!
struct StatsCard {
    label: String,
    value: String,
}

impl Component for StatsCard {
    fn show(&mut self, ui: &mut egui::Ui) {
        sushi::components::card(ui, |ui| {
            ui.label(&self.label);
            ui.heading(&self.value);
        });
    }
}

struct MyApp;
impl SushiApp for MyApp {
    fn name(&self) -> &str { "🍣 sushi - Easy Components" }
    fn update(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // 2. Use the view! macro for clean nesting
            view!(ui, {
                ui.heading("Dashboard");
                
                ui.horizontal(|ui| {
                    // 3. Render your custom components effortlessly
                    render(ui, StatsCard { label: "CPU".into(), value: "12%".into() });
                    render(ui, StatsCard { label: "RAM".into(), value: "1.2GB".into() });
                });
            });
        });
    }
}

fn main() -> sushi::eframe::Result<()> {
    run_app(MyApp)
}
