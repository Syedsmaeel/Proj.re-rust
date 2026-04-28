//! sushi — Sovereign React in Rust + Cyberpunk TUI helpers.
//!
//! Two layers:
//!   * `reconciler` — a real React-Fiber-style work loop with `use_state`,
//!     a deadline-aware scheduler, prop diffing, child reconciliation by
//!     index, and a pluggable `Host` trait so the same component tree can
//!     render to a debug sink, a TUI buffer, or anything you implement.
//!   * TUI helpers — thin wrappers around ratatui + crossterm so demo apps
//!     don't repeat boilerplate.

pub use crossterm;
pub use ratatui;

pub mod host;
pub mod reconciler;

pub use host::{DebugHost, Host, TuiHost};
pub use reconciler::fiber::{flags, ChildSpec, FiberId, Props, WorkTag};
pub use reconciler::hooks::{use_effect, use_state, Setter, UpdateQueue};
pub use reconciler::scheduler::Scheduler;
pub use reconciler::Reconciler;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge},
    Terminal,
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

pub fn loading_bar<'a>(label: &'a str, percent: u16) -> Gauge<'a> {
    Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(label))
        .gauge_style(
            Style::default()
                .fg(Color::Cyan)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .percent(percent.min(100))
}

pub trait DynamicSushiApp {
    fn title(&self) -> &str;
    fn tabs(&self) -> Vec<&str>;
    fn update(&mut self, key: crossterm::event::KeyCode) -> bool;
    fn render(
        &self,
        frame: &mut ratatui::Frame,
        area: ratatui::layout::Rect,
        active_tab: usize,
    );
}

pub fn render_snapshot<T: DynamicSushiApp>(
    app: &T,
    width: u16,
    height: u16,
) -> Result<String, anyhow::Error> {
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
            output.push_str(cell.symbol());
        }
        output.push('\n');
    }
    Ok(output)
}
