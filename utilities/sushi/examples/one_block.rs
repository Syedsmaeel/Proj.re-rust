use sushi::{sushi_app, SushiUiExt};

// This is the ENTIRE application.
// No structs, no traits, no manual main.
sushi_app! {
    name: "🍣 Sushi Ultra-Easy",
    state: {
        count: i32 = 0,
        name: String = "Sovereign".to_string(),
    },
    ui: |state, ui| {
        ui.sushi_header("One-Block App");
        
        ui.sushi_card(|ui| {
            ui.label(format!("Hello, {}!", state.name));
            ui.text_edit_singleline(&mut state.name);
            
            ui.add_space(10.0);
            
            if ui.sushi_button(format!("Clicked {} times", state.count)).clicked() {
                state.count += 1;
            }
            
            if ui.sushi_button("Reset").clicked() {
                state.count = 0;
            }
        });
    }
}
