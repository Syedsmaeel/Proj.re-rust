#[macro_export]
macro_rules! view {
    ($ui:expr, { $($content:tt)* }) => {
        $ui.vertical(|ui| {
            $($content)*
        })
    };
}

/// The "Ultra-Easy" One-Block App Macro
/// Removes the need for structs, traits, and main functions.
#[macro_export]
macro_rules! sushi_app {
    (
        name: $name:expr,
        state: { $($field:ident : $type:ty = $val:expr),* $(,)? },
        ui: |$state:ident, $ui:ident| $body:block
    ) => {
        struct AppState {
            $($field: $type),*
        }

        impl $crate::SushiApp for AppState {
            fn name(&self) -> &str { $name }
            fn update(&mut self, ctx: &$crate::egui::Context) {
                $crate::egui::CentralPanel::default().show(ctx, |$ui| {
                    let $state = self;
                    $body
                });
            }
        }

        fn main() -> $crate::eframe::Result<()> {
            $crate::run_app(AppState {
                $($field: $val),*
            })
        }
    };
}
