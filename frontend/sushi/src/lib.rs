use wasm_bindgen::prelude::*;
use web_sys::{window, Document, HtmlElement};

#[wasm_bindgen]
pub fn init_sushi(root_id: &str) -> Result<(), JsValue> {
    // Set up panic hook for better browser error messages
    console_error_panic_hook::set_once();
    
    let window = window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    let root = document.get_element_by_id(root_id)
        .ok_or_else(|| format!("Root element #{} not found", root_id))?;
    
    let el: HtmlElement = document.create_element("div")?.dyn_into()?;
    el.set_inner_html("<h1>🍣 sushi — Raw Rust Frontend</h1><p>The Sovereignty Stack UI is alive.</p>");
    
    root.append_child(&el)?;
    Ok(())
}

/// A simple Component trait for the Sushi framework
pub trait Component {
    fn render(&self) -> String;
}
