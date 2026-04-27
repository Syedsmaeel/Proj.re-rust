use sushi::{init_tui, restore_tui, DynamicSushiApp, render_snapshot, ratatui::{
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
}

impl DynamicSushiApp for StackDashboard {
    fn title(&self) -> &str { "🍣 sushi dynamic dashboard" }
    fn tabs(&self) -> Vec<&str> { vec!["Overview", "AI Console", "Security", "Logs"] }
    fn update(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char('q') => return true,
            KeyCode::Right | KeyCode::Tab => self.active_tab = (self.active_tab + 1) % self.tabs().len(),
            _ => {}
        }
        false
    }

    fn render(&self, f: &mut ratatui::Frame, area: sushi::ratatui::layout::Rect, active_tab: usize) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
            .split(area);

        let titles = self.tabs().iter().cloned().map(Line::from).collect::<Vec<_>>();
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title(self.title()))
            .select(active_tab);
        f.render_widget(tabs, chunks[0]);

        let content = Paragraph::new("\n  Sovereignty Stack: Online\n  Status: Ready for Inference")
            .block(Block::default().title(" System Status ").borders(Borders::ALL));
        f.render_widget(content, chunks[1]);

        let status = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(" Overall System Load "))
            .percent(self.progress);
        f.render_widget(status, chunks[2]);
    }
}

fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = std::env::args().collect();
    let mut app = StackDashboard { active_tab: 0, progress: 42 };

    if args.contains(&"--snapshot".to_string()) {
        println!("📸 Sushi Snapshot:\n");
        let snapshot = render_snapshot(&app, 60, 15)?;
        println!("{}", snapshot);
        return Ok(());
    }

    let mut terminal = init_tui()?;
    loop {
        terminal.draw(|f| app.render(f, f.size(), app.active_tab))?;
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if app.update(key.code) { break; }
            }
        }
        app.progress = (app.progress + 1) % 101;
    }
    restore_tui()?;
    Ok(())
}
