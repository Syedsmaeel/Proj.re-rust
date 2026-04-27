pub use eframe;
pub use eframe::egui;

pub mod macros;
pub mod theme;
pub mod components;

pub use theme::SushiTheme;
pub use components::SushiUiExt;

/// The core trait for a Sushi application
pub trait SushiApp {
    fn name(&self) -> &str;
    fn theme(&self) -> SushiTheme { SushiTheme::dark() }
    fn update(&mut self, ctx: &egui::Context);
}

pub fn run_app<T: SushiApp + 'static>(mut app: T) -> eframe::Result<()> {
    let name = app.name().to_string(); // Get name BEFORE moving app
    let theme = app.theme();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_transparent(true),
        ..Default::default()
    };
    
    eframe::run_native(
        &name,
        options,
        Box::new(move |cc| {
            // Apply the Sushi theme on startup
            theme.apply(&cc.egui_ctx);
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
