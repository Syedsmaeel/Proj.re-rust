#[macro_export]
macro_rules! view {
    ($ui:expr, { $($content:tt)* }) => {
        $ui.vertical(|ui| {
            $($content)*
        })
    };
}
