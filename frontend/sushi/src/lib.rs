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
