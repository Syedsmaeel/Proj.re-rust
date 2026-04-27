pub use eframe;
pub use eframe::egui;

pub mod theme;
pub mod components;

pub use theme::SushiTheme;

/// The core trait for a Sushi application
pub trait SushiApp {
    fn name(&self) -> &str;
    fn theme(&self) -> SushiTheme { SushiTheme::dark() }
    fn update(&mut self, ctx: &egui::Context);
}

pub fn run_app<T: SushiApp + 'static>(mut app: T) -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_transparent(true),
        ..Default::default()
    };
    
    eframe::run_native(
        app.name(),
        options,
        Box::new(|cc| {
            // Apply the Sushi theme on startup
            app.theme().apply(&cc.egui_ctx);
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
