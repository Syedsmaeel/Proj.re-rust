use sushi::{init_tui, restore_tui, DynamicSushiApp, ratatui::{
    widgets::{Block, Borders, Tabs, Paragraph, Gauge},
    layout::{Layout, Constraint, Direction},
    style::{Style, Color, Modifier},
    text::{Line, Span},
}};
use crossterm::event::{self, Event, KeyCode};
use std::time::Duration;

struct StackDashboard {
    active_tab: usize,
    progress: u16,
    log: Vec<String>,
}

impl DynamicSushiApp for StackDashboard {
    fn title(&self) -> &str { "🍣 sushi dynamic dashboard" }
    fn tabs(&self) -> Vec<&str> { vec!["Overview", "AI Console", "Security", "Logs"] }
    
    fn update(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char('q') => return true,
            KeyCode::Right | KeyCode::Tab => self.active_tab = (self.active_tab + 1) % self.tabs().len(),
            KeyCode::Left => self.active_tab = if self.active_tab == 0 { self.tabs().len() - 1 } else { self.active_tab - 1 },
            _ => {}
        }
        false
    }

    fn render(&self, f: &mut ratatui::Frame, area: sushi::ratatui::layout::Rect, active_tab: usize) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header/Tabs
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Status Bar
            ])
            .split(area);

        // 1. Render Tabs
        let titles: Vec<Line> = self.tabs().iter().cloned().map(Line::from).collect();
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title(self.title()))
            .select(active_tab)
            .highlight_style(Style::default().fg(Color::Rgb(255, 120, 0)).add_modifier(Modifier::BOLD));
        f.render_widget(tabs, chunks[0]);

        // 2. Render Content based on Active Tab
        match active_tab {
            0 => {
                let content = Paragraph::new("\n  Sovereignty Stack: Online\n  Active Pillars: 6\n  System Health: 100%")
                    .block(Block::default().title(" System Status ").borders(Borders::ALL));
                f.render_widget(content, chunks[1]);
            }
            1 => {
                let content = Paragraph::new("\n  AI Model: Phi-3-mini (GGUF)\n  Status: Ready for Inference\n  Backend: candle (Rust)")
                    .block(Block::default().title(" AI Console ").borders(Borders::ALL));
                f.render_widget(content, chunks[1]);
            }
            _ => {
                let content = Paragraph::new("\n  View not yet implemented...")
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(content, chunks[1]);
            }
        }

        // 3. Render Status Bar with Progress
        let status = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(" Overall System Load "))
            .gauge_style(Style::default().fg(Color::Rgb(255, 120, 0)))
            .percent(self.progress);
        f.render_widget(status, chunks[2]);
    }
}

fn main() -> Result<(), anyhow::Error> {
    let mut terminal = init_tui()?;
    let mut app = StackDashboard {
        active_tab: 0,
        progress: 42,
        log: vec!["Stack initialized.".to_string()],
    };

    loop {
        terminal.draw(|f| app.render(f, f.size(), app.active_tab))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if app.update(key.code) { break; }
            }
        }
        
        // Dynamic update
        app.progress = (app.progress + 1) % 101;
    }

    restore_tui()?;
    Ok(())
}
