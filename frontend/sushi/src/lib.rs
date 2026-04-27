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
            output.push_str(&cell.symbol);
        }
        output.push('\n');
    }
    Ok(output)
}
