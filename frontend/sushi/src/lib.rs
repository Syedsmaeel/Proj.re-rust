pub use ratatui;
pub use crossterm;

use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, Gauge, Paragraph},
    Terminal,
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

/// A standard Sushi Loading Bar component
pub fn loading_bar(title: &str, progress: u16) -> Gauge {
    Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(title))
        .gauge_style(ratatui::style::Style::default().fg(ratatui::style::Color::Rgb(255, 120, 0))) // Sushi Orange
        .percent(progress)
}
