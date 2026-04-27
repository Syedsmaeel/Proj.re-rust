pub use ratatui;
pub use crossterm;

use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, Gauge, Tabs, Paragraph},
    layout::{Layout, Constraint, Direction, Rect},
    Terminal,
    style::{Style, Color, Modifier},
    text::{Line, Span},
};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;

pub fn init_tui() -> Result<Terminal<CrosstermBackend<io::Stdout>>, anyhow::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

pub fn restore_tui() -> Result<(), anyhow::Error> {
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen)?;
    Ok(())
}

/// The core Dynamic App trait
pub trait DynamicSushiApp {
    fn title(&self) -> &str;
    fn tabs(&self) -> Vec<&str>;
    fn update(&mut self, key: KeyCode) -> bool; // returns true if should exit
    fn render(&self, frame: &mut ratatui::Frame, area: Rect, active_tab: usize);
}

/// NEW: Render a single frame to a string for debugging/testing
pub fn render_snapshot<T: DynamicSushiApp>(app: &T, width: u16, height: u16) -> Result<String, anyhow::Error> {
    use ratatui::backend::TestBackend;
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend)?;
    
    terminal.draw(|f| {
        app.render(f, f.size(), 0);
    })?;
    
    let mut output = String::new();
    let view = terminal.backend();
    for y in 0..height {
        for x in 0..width {
            let cell = view.buffer().get(x, y);
            output.push_str(&cell.symbol());
        }
        output.push('\n');
    }
    Ok(output)
}
pub mod reconciler;
pub mod hooks;
pub use hooks::{Hook, HookContext};

// Global state for the 'Current Component' (similar to how React handles hooks)
thread_local! {
    pub static HOOK_CTX: Arc<Mutex<Option<HookContext>>> = Arc::new(Mutex::new(None));
}

/// The 'use_state' Hook - Pure React Style
pub fn use_state<T: Any + Clone + Send>(initial: T) -> (T, impl Fn(T)) {
    // This is a simplified version for the demo
    let val = initial.clone();
    let setter = move |_new_val: T| {
        // In a real implementation, this would trigger a re-render
        println!("⚛️  React Hook: State updating...");
    };
    (val, setter)
}
