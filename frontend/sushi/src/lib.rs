pub use eframe;
use eframe::egui;

/// The core trait for a Sushi application
pub trait SushiApp {
    fn name(&self) -> &str;
    fn update(&mut self, ctx: &egui::Context);
}

/// The runner that launches a Sushi app natively
pub fn run_app<T: SushiApp + 'static>(mut app: T) -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        app.name(),
        options,
        Box::new(|_cc| {
            Box::new(SushiWrapper { app })
        }),
    )
}

struct SushiWrapper<T: SushiApp> {
    app: T,
}

impl<T: SushiApp> eframe::App for SushiWrapper<T> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.app.update(ctx);
    }
}
