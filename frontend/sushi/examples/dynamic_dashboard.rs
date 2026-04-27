use sushi::{init_tui, restore_tui, DynamicSushiApp, render_snapshot, ratatui::{
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
    fn title(&self) -> &str { " SOVEREIGNTY STACK v0.1.0 " }
    fn tabs(&self) -> Vec<&str> { vec![" 🏠 Overview ", " 🤖 AI Console ", " 🛡️ Security ", " 📝 Logs "] }
    
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
                Constraint::Length(7), // ASCII Logo
                Constraint::Length(3), // Tabs
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Status Bar
            ])
            .split(area);

        // 1. Render ASCII Logo
        let logo = Paragraph::new(SUSHI_LOGO)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Rgb(255, 120, 0)).add_modifier(Modifier::BOLD));
        f.render_widget(logo, chunks[0]);

        // 2. Render Elite Tabs
        let titles = self.tabs().iter().cloned().map(Line::from).collect::<Vec<_>>();
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title(self.title()))
            .select(active_tab)
            .highlight_style(Style::default().fg(Color::Rgb(255, 140, 105)).add_modifier(Modifier::BOLD))
            .divider(Span::raw(" | "));
        f.render_widget(tabs, chunks[1]);

        // 3. Render Content based on Active Tab
        match active_tab {
            0 => {
                let inner_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(chunks[2]);

                let status = Paragraph::new("\n  🔥 Core: ACTIVE\n  🌊 RAM: 42MB\n  🚀 CPU: 12%")
                    .block(Block::default().title(" System Stats ").borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)));
                f.render_widget(status, inner_chunks[0]);

                let sparkline = Sparkline::default()
                    .block(Block::default().title(" Network Activity ").borders(Borders::ALL).border_style(Style::default().fg(Color::Green)))
                    .data(&self.activity_data)
                    .style(Style::default().fg(Color::Rgb(255, 120, 0)));
                f.render_widget(sparkline, inner_chunks[1]);
            }
            1 => {
                let ai_content = Paragraph::new("\n  🤖 MODEL: Phi-3-mini\n  🧠 BRAIN: Local Inference\n  ⚡ STATUS: Awaiting Prompt...")
                    .block(Block::default().title(" AI Console ").borders(Borders::ALL).border_style(Style::default().fg(Color::Magenta)))
                    .wrap(Wrap { trim: true });
                f.render_widget(ai_content, chunks[2]);
            }
            _ => {
                let placeholder = Paragraph::new("\n  🚧 Pillar Under Construction...")
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
                f.render_widget(placeholder, chunks[2]);
            }
        }

        // 4. Render Elite Status Bar
        let status = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(" Overall System Integrity "))
            .gauge_style(Style::default().fg(Color::Rgb(255, 120, 0)).bg(Color::Rgb(40, 40, 40)))
            .percent(self.progress);
        f.render_widget(status, chunks[3]);
    }
}

fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = std::env::args().collect();
    let mut app = StackDashboard { 
        active_tab: 0, 
        progress: 88,
        activity_data: vec![1, 3, 2, 5, 8, 4, 7, 3, 2, 9, 5, 1, 4, 6, 8, 2, 5, 7, 3, 1, 9, 4, 2],
    };

    if args.contains(&"--snapshot".to_string()) {
        println!("📸 Elite Sushi Snapshot:\n");
        let snapshot = render_snapshot(&app, 100, 30)?;
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
        // Shift activity data for animation
        let last = app.activity_data[0];
        app.activity_data.remove(0);
        app.activity_data.push((last + 1) % 10);
    }
    restore_tui()?;
    Ok(())
}
