use sushi::{init_tui, restore_tui, loading_bar, ratatui::{widgets::{Block, Borders, Paragraph}, layout::{Layout, Constraint, Direction}}};
use std::time::{Duration, Instant};

fn main() -> Result<(), anyhow::Error> {
    let mut terminal = init_tui()?;
    let mut progress = 0;
    let start_time = Instant::now();

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(0),
                ].as_ref())
                .split(f.size());

            let header = Paragraph::new("🍣 sushi — Sovereign TUI Dashboard")
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(header, chunks[0]);

            // The "Load" Upgrade: Dynamic Loading Bar
            let bar = loading_bar("Loading Sovereignty Stack...", progress);
            f.render_widget(bar, chunks[1]);

            let info = Paragraph::new(format!("\n  Uptime: {:?}\n  Status: Running local LLM inference...", start_time.elapsed()))
                .block(Block::default().title(" System Log ").borders(Borders::ALL));
            f.render_widget(info, chunks[2]);
        })?;

        if crossterm::event::poll(Duration::from_millis(100))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                if key.code == crossterm::event::KeyCode::Char('q') {
                    break;
                }
            }
        }

        progress = (progress + 1) % 101;
    }

    restore_tui()?;
    Ok(())
}
