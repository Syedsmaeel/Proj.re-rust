use sushi::{init_tui, restore_tui, DynamicSushiApp, render_snapshot, Props, reconciler::{Reconciler, fiber::WorkTag}, ratatui::{
    widgets::{Block, Borders, Tabs, Paragraph, Gauge, Sparkline, Wrap},
    layout::{Layout, Constraint, Direction, Alignment},
    style::{Style, Color, Modifier},
    text::{Line, Span},
}};
use crossterm::event::{self, Event, KeyCode};
use std::time::Duration;

struct StackDashboard {
    active_tab: usize,
    progress: u16,
    activity_data: Vec<u64>,
    reconciler: Reconciler,
}

const SUSHI_LOGO: &str = r#"
   _____ _    _  _____ _    _ _____ 
  / ____| |  | |/ ____| |  | |_   _|
 | (___ | |  | | (___ | |__| | | |  
  \___ \| |  | |\___ \|  __  | | |  
  ____) | |__| |____) | |  | |_| |_ 
 |_____/ \____/|_____/|_|  |_|_____|
"#;

impl DynamicSushiApp for StackDashboard {
    fn title(&self) -> &str { " SOVEREIGNTY STACK v1.0.0 " }
    fn tabs(&self) -> Vec<&str> { vec![" 🏠 Overview ", " 🤖 AI Console ", " 🛡️ Security ", " 📝 Logs "] }
    
    fn update(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char('q') => return true,
            KeyCode::Right | KeyCode::Tab => {
                self.active_tab = (self.active_tab + 1) % self.tabs().len();
                // Trigger React-style Reconciler
                let _id = self.reconciler.create_fiber(WorkTag::FunctionComponent, Props::Text("TabSwitch".into()));
                println!("⚛️  Reconciler: Tab switched to #{}", self.active_tab);
            },
            _ => {}
        }
        false
    }

    fn render(&self, f: &mut ratatui::Frame, area: sushi::ratatui::layout::Rect, active_tab: usize) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7), // ASCII Logo
                Constraint::Length(3), // Tabs
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Status Bar
            ])
            .split(area);

        // Render UI...
        let logo = Paragraph::new(SUSHI_LOGO)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Rgb(255, 120, 0)).add_modifier(Modifier::BOLD));
        f.render_widget(logo, chunks[0]);

        let titles: Vec<Line> = self.tabs().iter().cloned().map(Line::from).collect();
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title(self.title()))
            .select(active_tab)
            .highlight_style(Style::default().fg(Color::Rgb(255, 140, 105)).add_modifier(Modifier::BOLD));
        f.render_widget(tabs, chunks[1]);

        let content = Paragraph::new("\n  🚀 RECONCILER: ACTIVE\n  ⚛️  MODE: SOVEREIGN REACT\n  ⚡ ENGINE: PURE RUST")
            .block(Block::default().title(" System Brain ").borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)));
        f.render_widget(content, chunks[2]);

        let status = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(" Overall System Integrity "))
            .gauge_style(Style::default().fg(Color::Rgb(255, 120, 0)))
            .percent(self.progress);
        f.render_widget(status, chunks[3]);
    }
}

fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = std::env::args().collect();
    let app = StackDashboard { 
        active_tab: 0, 
        progress: 100,
        activity_data: vec![1, 2, 3],
        reconciler: Reconciler::new(),
    };

    if args.contains(&"--snapshot".to_string()) {
        println!("📸 Final Sushi React-TUI Snapshot:\n");
        let snapshot = render_snapshot(&app, 100, 25)?;
        println!("{}", snapshot);
        return Ok(());
    }

    let mut terminal = init_tui()?;
    // (Standard loop logic omitted for brevity in snapshot mode)
    restore_tui()?;
    Ok(())
}
