//! Bash++ TUI — full terminal OS shell renderer
//! Tabs, split panes, widgets, icons, colors, borders
extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;
use crate::bashpp::parser::{PaneSpec, SplitDir, BorderStyle, TabSpec, WidgetKind};

/// ANSI color codes
pub mod color {
    pub const RESET:   &str = "\x1b[0m";
    pub const BOLD:    &str = "\x1b[1m";
    pub const DIM:     &str = "\x1b[2m";
    pub const BLACK:   &str = "\x1b[30m";
    pub const RED:     &str = "\x1b[31m";
    pub const GREEN:   &str = "\x1b[32m";
    pub const YELLOW:  &str = "\x1b[33m";
    pub const BLUE:    &str = "\x1b[34m";
    pub const MAGENTA: &str = "\x1b[35m";
    pub const CYAN:    &str = "\x1b[36m";
    pub const WHITE:   &str = "\x1b[37m";
    pub const BG_BLACK: &str = "\x1b[40m";
    pub const BG_BLUE:  &str = "\x1b[44m";
    pub const BG_CYAN:  &str = "\x1b[46m";
    // 256-color
    pub fn fg256(n: u8) -> alloc::string::String { alloc::format!("\x1b[38;5;{}m", n) }
    pub fn bg256(n: u8) -> alloc::string::String { alloc::format!("\x1b[48;5;{}m", n) }
    // True color
    pub fn fg_rgb(r: u8, g: u8, b: u8) -> alloc::string::String { alloc::format!("\x1b[38;2;{};{};{}m", r, g, b) }
    pub fn bg_rgb(r: u8, g: u8, b: u8) -> alloc::string::String { alloc::format!("\x1b[48;2;{};{};{}m", r, g, b) }
    pub fn named(name: &str) -> &'static str {
        match name {
            "red"=>RED,"green"=>GREEN,"yellow"=>YELLOW,"blue"=>BLUE,
            "magenta"=>MAGENTA,"cyan"=>CYAN,"white"=>WHITE,"black"=>BLACK,
            _=>RESET,
        }
    }
}

/// Border drawing characters
pub struct Border {
    pub tl: &'static str, pub tr: &'static str,
    pub bl: &'static str, pub br: &'static str,
    pub h:  &'static str, pub v:  &'static str,
    pub lt: &'static str, pub rt: &'static str,
    pub tt: &'static str, pub bt: &'static str,
}

impl Border {
    pub fn from_style(s: &BorderStyle) -> Self {
        match s {
            BorderStyle::None    => Self { tl:"",tr:"",bl:"",br:"",h:"",v:"",lt:"",rt:"",tt:"",bt:"" },
            BorderStyle::Single  => Self { tl:"┌",tr:"┐",bl:"└",br:"┘",h:"─",v:"│",lt:"├",rt:"┤",tt:"┬",bt:"┴" },
            BorderStyle::Double  => Self { tl:"╔",tr:"╗",bl:"╚",br:"╝",h:"═",v:"║",lt:"╠",rt:"╣",tt:"╦",bt:"╩" },
            BorderStyle::Rounded => Self { tl:"╭",tr:"╮",bl:"╰",br:"╯",h:"─",v:"│",lt:"├",rt:"┤",tt:"┬",bt:"┴" },
            BorderStyle::Thick   => Self { tl:"┏",tr:"┓",bl:"┗",br:"┛",h:"━",v:"┃",lt:"┣",rt:"┫",tt:"┳",bt:"┻" },
        }
    }

    pub fn draw_box(&self, width: usize, title: Option<&str>) -> Vec<String> {
        let mut lines = Vec::new();
        let inner = width.saturating_sub(2);
        // Top border with optional title
        if let Some(t) = title {
            let title_str = alloc::format!(" {} ", t);
            let pad = inner.saturating_sub(title_str.len());
            let lpad = pad / 2;
            let rpad = pad - lpad;
            lines.push(alloc::format!("{}{}{}{}{}{}",
                self.tl,
                self.h.repeat(lpad),
                title_str,
                self.h.repeat(rpad),
                self.tr,
                color::RESET,
            ));
        } else {
            lines.push(alloc::format!("{}{}{}", self.tl, self.h.repeat(inner), self.tr));
        }
        lines
    }

    pub fn side_line(&self, content: &str, width: usize) -> String {
        let inner = width.saturating_sub(2);
        let content_len = content.chars().count();
        let pad = inner.saturating_sub(content_len);
        alloc::format!("{} {}{}{}", self.v, content, " ".repeat(pad), self.v)
    }

    pub fn bottom(&self, width: usize) -> String {
        let inner = width.saturating_sub(2);
        alloc::format!("{}{}{}", self.bl, self.h.repeat(inner), self.br)
    }
}

/// A rendered cell — one row of terminal text
#[derive(Debug, Clone)]
pub struct Cell {
    pub text:  String,
    pub color: Option<String>,
    pub bold:  bool,
    pub dim:   bool,
}

impl Cell {
    pub fn plain(text: &str) -> Self {
        Self { text: String::from(text), color: None, bold: false, dim: false }
    }
    pub fn colored(text: &str, color: &str) -> Self {
        Self { text: String::from(text), color: Some(String::from(color)), bold: false, dim: false }
    }
    pub fn render(&self) -> String {
        let mut s = String::new();
        if self.bold  { s.push_str(color::BOLD); }
        if self.dim   { s.push_str(color::DIM); }
        if let Some(c) = &self.color { s.push_str(c); }
        s.push_str(&self.text);
        s.push_str(color::RESET);
        s
    }
}

/// Available built-in themes
#[derive(Debug, Clone, PartialEq)]
pub enum Theme {
    Default,
    Dracula,
    Nord,
    Gruvbox,
    Solarized,
    Catppuccin,
    Custom(String),
}

impl Theme {
    pub fn from_name(name: &str) -> Self {
        match name {
            "dracula"    => Self::Dracula,
            "nord"       => Self::Nord,
            "gruvbox"    => Self::Gruvbox,
            "solarized"  => Self::Solarized,
            "catppuccin" => Self::Catppuccin,
            other        => Self::Custom(String::from(other)),
        }
    }

    pub fn accent(&self) -> &'static str {
        match self {
            Self::Dracula    => "\x1b[38;5;141m", // purple
            Self::Nord       => "\x1b[38;5;111m", // frost blue
            Self::Gruvbox    => "\x1b[38;5;214m", // orange
            Self::Solarized  => "\x1b[38;5;37m",  // teal
            Self::Catppuccin => "\x1b[38;5;183m", // mauve
            _                => color::CYAN,
        }
    }

    pub fn bg(&self) -> &'static str {
        match self {
            Self::Dracula    => "\x1b[48;5;236m",
            Self::Nord       => "\x1b[48;5;238m",
            Self::Gruvbox    => "\x1b[48;5;235m",
            Self::Solarized  => "\x1b[48;5;234m",
            Self::Catppuccin => "\x1b[48;5;237m",
            _                => color::BG_BLACK,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Default    => "default",
            Self::Dracula    => "dracula",
            Self::Nord       => "nord",
            Self::Gruvbox    => "gruvbox",
            Self::Solarized  => "solarized",
            Self::Catppuccin => "catppuccin",
            Self::Custom(n)  => n,
        }
    }
}

/// A pane — one region of the terminal
#[derive(Debug, Clone)]
pub struct Pane {
    pub id:      usize,
    pub title:   Option<String>,
    pub cells:   Vec<Cell>,
    pub split:   Option<Box<PaneSplit>>,
    pub border:  BorderStyle,
    pub width:   usize,
    pub height:  usize,
    pub focused: bool,
}

#[derive(Debug, Clone)]
pub struct PaneSplit {
    pub dir:   SplitDir,
    pub ratio: u8,
    pub left:  Pane,
    pub right: Pane,
}

impl Pane {
    pub fn new(id: usize, width: usize, height: usize) -> Self {
        Self { id, title: None, cells: Vec::new(), split: None,
               border: BorderStyle::Rounded, width, height, focused: false }
    }

    pub fn with_title(mut self, t: &str) -> Self { self.title = Some(String::from(t)); self }
    pub fn with_border(mut self, b: BorderStyle) -> Self { self.border = b; self }
    pub fn focused(mut self) -> Self { self.focused = true; self }

    pub fn push(&mut self, cell: Cell) { self.cells.push(cell); }

    /// Render this pane to lines of text
    pub fn render(&self, theme: &Theme) -> Vec<String> {
        let mut lines = Vec::new();
        let border = Border::from_style(&self.border);
        let accent = if self.focused { theme.accent() } else { color::DIM };

        // Top border
        let top = if let Some(t) = &self.title {
            let title = alloc::format!(" {} ", t);
            let inner = self.width.saturating_sub(2);
            let pad = inner.saturating_sub(title.len());
            alloc::format!("{}{}{}{}{}{}{}",
                accent, border.tl,
                border.h.repeat(pad/2),
                title,
                border.h.repeat(pad - pad/2),
                border.tr, color::RESET)
        } else {
            alloc::format!("{}{}{}{}{}",
                accent, border.tl,
                border.h.repeat(self.width.saturating_sub(2)),
                border.tr, color::RESET)
        };
        lines.push(top);

        // Content lines
        for cell in &self.cells {
            lines.push(border.side_line(&cell.render(), self.width));
        }

        // Empty padding
        let content_rows = self.height.saturating_sub(2);
        for _ in self.cells.len()..content_rows {
            lines.push(border.side_line("", self.width));
        }

        // Bottom border
        lines.push(alloc::format!("{}{}{}{}{}",
            accent, border.bl,
            border.h.repeat(self.width.saturating_sub(2)),
            border.br, color::RESET));

        lines
    }
}

/// A tab in the tab bar
#[derive(Debug, Clone)]
pub struct Tab {
    pub name:    String,
    pub icon:    Option<String>,
    pub panes:   Vec<Pane>,
    pub active:  bool,
}

impl Tab {
    pub fn new(name: &str) -> Self {
        Self { name: String::from(name), icon: None, panes: Vec::new(), active: false }
    }

    pub fn with_icon(mut self, icon: &str) -> Self { self.icon = Some(String::from(icon)); self }
    pub fn active(mut self) -> Self { self.active = true; self }

    pub fn add_pane(&mut self, pane: Pane) { self.panes.push(pane); }

    pub fn render_tab_label(&self, theme: &Theme) -> String {
        let icon_str = self.icon.as_deref().unwrap_or("");
        let label = alloc::format!(" {}{} ", icon_str, self.name);
        if self.active {
            alloc::format!("{}{}{}{}", theme.accent(), color::BOLD, label, color::RESET)
        } else {
            alloc::format!("{}{}{}", color::DIM, label, color::RESET)
        }
    }
}

/// Built-in widget renderer
#[derive(Debug, Clone)]
pub struct Widget {
    pub kind:   WidgetKind,
    pub width:  usize,
    pub height: usize,
}

impl Widget {
    pub fn new(kind: WidgetKind, width: usize, height: usize) -> Self {
        Self { kind, width, height }
    }

    pub fn render(&self, theme: &Theme) -> Vec<String> {
        let border = Border::from_style(&BorderStyle::Rounded);
        match &self.kind {
            WidgetKind::Clock => {
                let mut lines = Vec::new();
                lines.push(alloc::format!("{}{}{}{}{}",
                    theme.accent(), border.tl,
                    border.h.repeat(self.width.saturating_sub(2)),
                    border.tr, color::RESET));
                lines.push(border.side_line(" 🕐 Clock ", self.width));
                lines.push(border.side_line("  00:00:00  ", self.width));
                lines.push(alloc::format!("{}{}{}{}",
                    theme.accent(), border.bl,
                    border.h.repeat(self.width.saturating_sub(2)),
                    border.br));
                lines
            }
            WidgetKind::ProcessViewer => {
                let mut lines = Vec::new();
                lines.push(alloc::format!("{}{}{}{}{}",
                    theme.accent(), border.tl,
                    border.h.repeat(self.width.saturating_sub(2)),
                    border.tr, color::RESET));
                lines.push(border.side_line(" ⚙ Processes ", self.width));
                lines.push(border.side_line("  PID   CPU   MEM   NAME", self.width));
                lines.push(border.side_line("  1     0.0%  4MB   init", self.width));
                lines.push(border.side_line("  2     1.2%  12MB  net-sk", self.width));
                lines.push(alloc::format!("{}{}{}{}",
                    theme.accent(), border.bl,
                    border.h.repeat(self.width.saturating_sub(2)),
                    border.br));
                lines
            }
            WidgetKind::FileTree { path } => {
                let mut lines = Vec::new();
                let title = alloc::format!(" 📁 {} ", path);
                lines.push(alloc::format!("{}{}{}{}{}",
                    theme.accent(), border.tl,
                    border.h.repeat(self.width.saturating_sub(2)),
                    border.tr, color::RESET));
                lines.push(border.side_line(&title, self.width));
                lines.push(border.side_line("  ├── src/", self.width));
                lines.push(border.side_line("  │   ├── main.rs", self.width));
                lines.push(border.side_line("  │   └── lib.rs", self.width));
                lines.push(border.side_line("  └── Cargo.toml", self.width));
                lines.push(alloc::format!("{}{}{}{}",
                    theme.accent(), border.bl,
                    border.h.repeat(self.width.saturating_sub(2)),
                    border.br));
                lines
            }
            WidgetKind::StatusBar => {
                alloc::vec![alloc::format!("{}{}  🦀 bashpp  |  timux  |  ring-2  {}{}",
                    theme.bg(), theme.accent(), color::RESET, color::RESET)]
            }
            WidgetKind::CommandPalette => {
                let mut lines = Vec::new();
                lines.push(alloc::format!("{}{}{}{}{}",
                    theme.accent(), border.tl,
                    border.h.repeat(self.width.saturating_sub(2)),
                    border.tr, color::RESET));
                lines.push(border.side_line(" 🔍 Command Palette ", self.width));
                lines.push(border.side_line("  > _", self.width));
                lines.push(border.side_line("  pane  tab  widget  theme", self.width));
                lines.push(alloc::format!("{}{}{}{}",
                    theme.accent(), border.bl,
                    border.h.repeat(self.width.saturating_sub(2)),
                    border.br));
                lines
            }
            WidgetKind::Editor { file } => {
                let title = alloc::format!(" ✏ {} ", file.as_deref().unwrap_or("untitled"));
                let mut lines = Vec::new();
                lines.push(alloc::format!("{}{}{}{}{}",
                    theme.accent(), border.tl,
                    border.h.repeat(self.width.saturating_sub(2)),
                    border.tr, color::RESET));
                lines.push(border.side_line(&title, self.width));
                lines.push(border.side_line("  1  ", self.width));
                lines.push(border.side_line("  2  ", self.width));
                lines.push(alloc::format!("{}{}{}{}",
                    theme.accent(), border.bl,
                    border.h.repeat(self.width.saturating_sub(2)),
                    border.br));
                lines
            }
            WidgetKind::Custom(name) => {
                alloc::vec![alloc::format!("[widget: {}]", name)]
            }
        }
    }
}

/// The full TUI shell state
pub struct TuiShell {
    pub tabs:         Vec<Tab>,
    pub active_tab:   usize,
    pub theme:        Theme,
    pub width:        usize,
    pub height:       usize,
    pub prompt:       String,
    pub history:      Vec<String>,
    pub status:       String,
}

impl TuiShell {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: 0,
            theme: Theme::Default,
            width,
            height,
            prompt: String::from("bashpp❯ "),
            history: Vec::new(),
            status: String::from("ready"),
        }
    }

    pub fn set_theme(&mut self, name: &str) { self.theme = Theme::from_name(name); }

    pub fn add_tab(&mut self, tab: Tab) { self.tabs.push(tab); }

    pub fn switch_tab(&mut self, idx: usize) {
        if idx < self.tabs.len() {
            for (i, t) in self.tabs.iter_mut().enumerate() {
                t.active = i == idx;
            }
            self.active_tab = idx;
        }
    }

    /// Render the full TUI to a list of terminal lines
    pub fn render(&self) -> Vec<String> {
        let mut out = Vec::new();
        let theme = &self.theme;

        // ── Tab bar ────────────────────────────────────────────────────────
        let tab_bar: String = self.tabs.iter()
            .map(|t| t.render_tab_label(theme))
            .collect::<Vec<_>>()
            .join(&alloc::format!("{}│{}", color::DIM, color::RESET));
        out.push(alloc::format!("{}╭{}╮",
            theme.accent(),
            "─".repeat(self.width.saturating_sub(2))));
        out.push(alloc::format!("{}│{} {}{}│",
            theme.accent(), color::RESET, tab_bar, theme.accent()));
        out.push(alloc::format!("{}╰{}╯", theme.accent(),
            "─".repeat(self.width.saturating_sub(2))));

        // ── Active tab content ─────────────────────────────────────────────
        if let Some(tab) = self.tabs.get(self.active_tab) {
            for pane in &tab.panes {
                for line in pane.render(theme) {
                    out.push(line);
                }
            }
        }

        // ── Status bar ─────────────────────────────────────────────────────
        let status = Widget::new(WidgetKind::StatusBar, self.width, 1);
        for line in status.render(theme) { out.push(line); }

        // ── Prompt ────────────────────────────────────────────────────────
        out.push(alloc::format!("{}{}{}{} ",
            theme.accent(), color::BOLD, self.prompt, color::RESET));

        out
    }

    pub fn push_history(&mut self, cmd: &str) { self.history.push(String::from(cmd)); }
    pub fn set_status(&mut self, s: &str) { self.status = String::from(s); }
}
