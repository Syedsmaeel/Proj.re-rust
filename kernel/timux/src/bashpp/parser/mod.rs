//! Bash++ Parser — builds an AST from tokens
extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;
use crate::bashpp::lexer::{Token, TokenKind};

/// A Bash++ value
#[derive(Debug, Clone)]
pub enum Value {
    Str(String),
    Int(i64),
    Bool(bool),
    List(Vec<Value>),
    Null,
}

/// A redirect operation
#[derive(Debug, Clone)]
pub enum Redirect {
    Out(String),        // > file
    Append(String),     // >> file
    In(String),         // < file
    Heredoc(String),    // << EOF
}

/// A single command with args and redirects
#[derive(Debug, Clone)]
pub struct Command {
    pub name:      String,
    pub args:      Vec<Expr>,
    pub redirects: Vec<Redirect>,
    pub background: bool,
}

/// An expression
#[derive(Debug, Clone)]
pub enum Expr {
    Lit(Value),
    Var(String),                              // $name
    Interp(String),                           // "${name}"
    SubShell(Vec<Ast>),                       // $(...)
    Cmd(Command),
    Assign { name: String, val: Box<Expr> },  // let x := val
    BinOp { op: BinOp, lhs: Box<Expr>, rhs: Box<Expr> },
    Pipeline(Vec<Expr>),                      // a | b | c
    And(Box<Expr>, Box<Expr>),                // a && b
    Or(Box<Expr>, Box<Expr>),                 // a || b
    Redirect { expr: Box<Expr>, redir: Redirect },
    // TUI exprs
    PaneExpr(PaneSpec),
    TabExpr(TabSpec),
    WidgetExpr(WidgetKind),
    RenderExpr { text: String, color: Option<String> },
    IconExpr(String),
    ThemeExpr(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp { Eq, Ne, Lt, Gt, Le, Ge, Add, Sub, Mul, Div }

/// TUI pane specification
#[derive(Debug, Clone)]
pub struct PaneSpec {
    pub split:    SplitDir,
    pub ratio:    u8,        // percent for first pane (default 50)
    pub children: Vec<Ast>,
    pub border:   BorderStyle,
    pub title:    Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SplitDir { Horizontal, Vertical }

#[derive(Debug, Clone, PartialEq)]
pub enum BorderStyle { None, Single, Double, Rounded, Thick }

/// TUI tab specification  
#[derive(Debug, Clone)]
pub struct TabSpec {
    pub name:     String,
    pub icon:     Option<String>,
    pub children: Vec<Ast>,
}

/// Built-in widgets
#[derive(Debug, Clone)]
pub enum WidgetKind {
    Clock,
    FileTree { path: String },
    ProcessViewer,
    Editor { file: Option<String> },
    CommandPalette,
    StatusBar,
    Custom(String),
}

/// An AST node
#[derive(Debug, Clone)]
pub enum Ast {
    Expr(Expr),
    If { cond: Box<Expr>, then: Vec<Ast>, else_: Option<Vec<Ast>> },
    For { var: String, iter: Box<Expr>, body: Vec<Ast> },
    While { cond: Box<Expr>, body: Vec<Ast> },
    Loop { body: Vec<Ast> },
    Match { val: Box<Expr>, arms: Vec<(Expr, Vec<Ast>)> },
    FnDef { name: String, params: Vec<String>, body: Vec<Ast> },
    Return(Option<Box<Expr>>),
    Break,
    Continue,
    Export(String, Option<Box<Expr>>),
    Unset(String),
    Source(String),
    Block(Vec<Ast>),
}

pub struct Parser {
    tokens: Vec<Token>,
    pos:    usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self { Self { tokens, pos: 0 } }

    fn peek(&self) -> &TokenKind { &self.tokens.get(self.pos).map(|t| &t.kind).unwrap_or(&TokenKind::Eof) }

    fn advance(&mut self) -> &TokenKind {
        let k = &self.tokens.get(self.pos).map(|t| &t.kind).unwrap_or(&TokenKind::Eof);
        if self.pos < self.tokens.len() { self.pos += 1; }
        k
    }

    fn skip_newlines(&mut self) {
        while matches!(self.peek(), TokenKind::Newline | TokenKind::Semicolon) { self.advance(); }
    }

    pub fn parse(&mut self) -> Vec<Ast> {
        let mut nodes = Vec::new();
        loop {
            self.skip_newlines();
            if matches!(self.peek(), TokenKind::Eof) { break; }
            if let Some(node) = self.parse_stmt() { nodes.push(node); }
        }
        nodes
    }

    fn parse_stmt(&mut self) -> Option<Ast> {
        match self.peek().clone() {
            TokenKind::KwIf       => Some(self.parse_if()),
            TokenKind::KwFor      => Some(self.parse_for()),
            TokenKind::KwWhile    => Some(self.parse_while()),
            TokenKind::KwLoop     => Some(self.parse_loop()),
            TokenKind::KwFn       => Some(self.parse_fn()),
            TokenKind::KwReturn   => { self.advance(); Some(Ast::Return(None)) }
            TokenKind::KwBreak    => { self.advance(); Some(Ast::Break) }
            TokenKind::KwContinue => { self.advance(); Some(Ast::Continue) }
            TokenKind::KwExport   => Some(self.parse_export()),
            TokenKind::KwUnset    => Some(self.parse_unset()),
            TokenKind::KwSource   => Some(self.parse_source()),
            TokenKind::KwMatch    => Some(self.parse_match()),
            _                     => Some(Ast::Expr(self.parse_expr())),
        }
    }

    fn parse_if(&mut self) -> Ast {
        self.advance(); // consume 'if'
        let cond = Box::new(self.parse_expr());
        self.skip_newlines();
        // consume 'then' or '{'
        if matches!(self.peek(), TokenKind::KwThen | TokenKind::LBrace) { self.advance(); }
        let then = self.parse_block_until(&[TokenKind::KwFi, TokenKind::KwElse, TokenKind::RBrace]);
        let else_ = if matches!(self.peek(), TokenKind::KwElse) {
            self.advance();
            if matches!(self.peek(), TokenKind::LBrace) { self.advance(); }
            Some(self.parse_block_until(&[TokenKind::KwFi, TokenKind::RBrace]))
        } else { None };
        if matches!(self.peek(), TokenKind::KwFi | TokenKind::RBrace) { self.advance(); }
        Ast::If { cond, then, else_ }
    }

    fn parse_for(&mut self) -> Ast {
        self.advance(); // 'for'
        let var = match self.advance().clone() {
            TokenKind::Word(w) => w,
            _ => String::from("_"),
        };
        // consume 'in'
        if matches!(self.peek(), TokenKind::KwIn) { self.advance(); }
        let iter = Box::new(self.parse_expr());
        self.skip_newlines();
        if matches!(self.peek(), TokenKind::KwDo | TokenKind::LBrace) { self.advance(); }
        let body = self.parse_block_until(&[TokenKind::KwDone, TokenKind::RBrace]);
        if matches!(self.peek(), TokenKind::KwDone | TokenKind::RBrace) { self.advance(); }
        Ast::For { var, iter, body }
    }

    fn parse_while(&mut self) -> Ast {
        self.advance();
        let cond = Box::new(self.parse_expr());
        self.skip_newlines();
        if matches!(self.peek(), TokenKind::KwDo | TokenKind::LBrace) { self.advance(); }
        let body = self.parse_block_until(&[TokenKind::KwDone, TokenKind::RBrace]);
        if matches!(self.peek(), TokenKind::KwDone | TokenKind::RBrace) { self.advance(); }
        Ast::While { cond, body }
    }

    fn parse_loop(&mut self) -> Ast {
        self.advance();
        if matches!(self.peek(), TokenKind::LBrace) { self.advance(); }
        let body = self.parse_block_until(&[TokenKind::RBrace]);
        if matches!(self.peek(), TokenKind::RBrace) { self.advance(); }
        Ast::Loop { body }
    }

    fn parse_fn(&mut self) -> Ast {
        self.advance(); // 'fn'
        let name = match self.advance().clone() {
            TokenKind::Word(w) => w,
            _ => String::from("anon"),
        };
        let mut params = alloc::vec::Vec::new();
        if matches!(self.peek(), TokenKind::LParen) {
            self.advance();
            while !matches!(self.peek(), TokenKind::RParen | TokenKind::Eof) {
                if let TokenKind::Word(p) = self.advance().clone() { params.push(p); }
                if matches!(self.peek(), TokenKind::Comma) { self.advance(); }
            }
            if matches!(self.peek(), TokenKind::RParen) { self.advance(); }
        }
        if matches!(self.peek(), TokenKind::LBrace) { self.advance(); }
        let body = self.parse_block_until(&[TokenKind::RBrace]);
        if matches!(self.peek(), TokenKind::RBrace) { self.advance(); }
        Ast::FnDef { name, params, body }
    }

    fn parse_match(&mut self) -> Ast {
        self.advance();
        let val = Box::new(self.parse_expr());
        if matches!(self.peek(), TokenKind::LBrace) { self.advance(); }
        let mut arms = Vec::new();
        while !matches!(self.peek(), TokenKind::RBrace | TokenKind::Eof) {
            self.skip_newlines();
            let pat = self.parse_expr();
            if matches!(self.peek(), TokenKind::FatArrow) { self.advance(); }
            if matches!(self.peek(), TokenKind::LBrace) { self.advance(); }
            let body = self.parse_block_until(&[TokenKind::RBrace, TokenKind::Comma]);
            if matches!(self.peek(), TokenKind::RBrace | TokenKind::Comma) { self.advance(); }
            arms.push((pat, body));
            self.skip_newlines();
        }
        if matches!(self.peek(), TokenKind::RBrace) { self.advance(); }
        Ast::Match { val, arms }
    }

    fn parse_export(&mut self) -> Ast {
        self.advance();
        let name = match self.advance().clone() { TokenKind::Word(w)=>w, _=>String::new() };
        let val = if matches!(self.peek(), TokenKind::Equals) {
            self.advance();
            Some(Box::new(self.parse_expr()))
        } else { None };
        Ast::Export(name, val)
    }

    fn parse_unset(&mut self) -> Ast {
        self.advance();
        let name = match self.advance().clone() { TokenKind::Word(w)=>w, _=>String::new() };
        Ast::Unset(name)
    }

    fn parse_source(&mut self) -> Ast {
        self.advance();
        let path = match self.advance().clone() {
            TokenKind::Word(w)|TokenKind::StringLit(w) => w,
            _ => String::new(),
        };
        Ast::Source(path)
    }

    fn parse_block_until(&mut self, stops: &[TokenKind]) -> Vec<Ast> {
        let mut nodes = Vec::new();
        loop {
            self.skip_newlines();
            if stops.iter().any(|s| self.peek() == s) || matches!(self.peek(), TokenKind::Eof) {
                break;
            }
            if let Some(n) = self.parse_stmt() { nodes.push(n); }
        }
        nodes
    }

    fn parse_expr(&mut self) -> Expr {
        self.parse_pipeline()
    }

    fn parse_pipeline(&mut self) -> Expr {
        let first = self.parse_and_or();
        if matches!(self.peek(), TokenKind::Pipe) {
            let mut parts = alloc::vec![first];
            while matches!(self.peek(), TokenKind::Pipe) {
                self.advance();
                parts.push(self.parse_and_or());
            }
            return Expr::Pipeline(parts);
        }
        first
    }

    fn parse_and_or(&mut self) -> Expr {
        let mut lhs = self.parse_primary();
        loop {
            match self.peek().clone() {
                TokenKind::DoubleAmp => { self.advance(); let rhs=self.parse_primary(); lhs=Expr::And(Box::new(lhs),Box::new(rhs)); }
                TokenKind::DoublePipe => { self.advance(); let rhs=self.parse_primary(); lhs=Expr::Or(Box::new(lhs),Box::new(rhs)); }
                _ => break,
            }
        }
        lhs
    }

    fn parse_primary(&mut self) -> Expr {
        match self.peek().clone() {
            TokenKind::KwLet => {
                self.advance();
                let name = match self.advance().clone() { TokenKind::Word(w)=>w, _=>String::new() };
                if matches!(self.peek(), TokenKind::Walrus|TokenKind::Equals) { self.advance(); }
                let val = Box::new(self.parse_expr());
                Expr::Assign { name, val }
            }
            TokenKind::KwPane => { self.advance(); Expr::PaneExpr(self.parse_pane()) }
            TokenKind::KwTab  => { self.advance(); Expr::TabExpr(self.parse_tab()) }
            TokenKind::KwWidget => { self.advance(); Expr::WidgetExpr(self.parse_widget()) }
            TokenKind::KwRender => { self.advance(); self.parse_render() }
            TokenKind::KwIcon => { self.advance();
                let icon = match self.advance().clone() { TokenKind::StringLit(s)=>s, TokenKind::Word(w)=>w, _=>String::new() };
                Expr::IconExpr(icon)
            }
            TokenKind::KwTheme => { self.advance();
                let name = match self.advance().clone() { TokenKind::Word(w)=>w, TokenKind::StringLit(s)=>s, _=>String::from("default") };
                Expr::ThemeExpr(name)
            }
            TokenKind::Dollar => {
                self.advance();
                let name = match self.advance().clone() { TokenKind::Word(w)=>w, _=>String::new() };
                Expr::Var(name)
            }
            TokenKind::DollarLBrace => {
                self.advance();
                let name = match self.advance().clone() { TokenKind::Word(w)=>w, _=>String::new() };
                if matches!(self.peek(), TokenKind::RBrace) { self.advance(); }
                Expr::Interp(name)
            }
            TokenKind::StringLit(s) => { let s=s.clone(); self.advance(); Expr::Lit(Value::Str(s)) }
            TokenKind::RawString(s) => { let s=s.clone(); self.advance(); Expr::Lit(Value::Str(s)) }
            TokenKind::Number(n) => { let nv=n; self.advance(); Expr::Lit(Value::Int(nv)) }
            TokenKind::Bool(b) => { let bv=b; self.advance(); Expr::Lit(Value::Bool(bv)) }
            TokenKind::Word(w) => {
                let name = w.clone(); self.advance();
                let mut args = Vec::new();
                let mut bg = false;
                while !matches!(self.peek(),
                    TokenKind::Newline|TokenKind::Semicolon|TokenKind::Eof|
                    TokenKind::Pipe|TokenKind::DoubleAmp|TokenKind::DoublePipe|
                    TokenKind::RBrace|TokenKind::KwThen|TokenKind::KwDo|
                    TokenKind::KwDone|TokenKind::KwFi|TokenKind::KwElse) {
                    if matches!(self.peek(), TokenKind::Amp) { self.advance(); bg=true; break; }
                    args.push(self.parse_atom());
                }
                Expr::Cmd(Command { name, args, redirects: Vec::new(), background: bg })
            }
            _ => { self.advance(); Expr::Lit(Value::Null) }
        }
    }

    fn parse_atom(&mut self) -> Expr {
        match self.peek().clone() {
            TokenKind::StringLit(s) => { let s=s.clone(); self.advance(); Expr::Lit(Value::Str(s)) }
            TokenKind::RawString(s) => { let s=s.clone(); self.advance(); Expr::Lit(Value::Str(s)) }
            TokenKind::Number(n)    => { let nv=n; self.advance(); Expr::Lit(Value::Int(nv)) }
            TokenKind::Word(w)      => { let w=w.clone(); self.advance(); Expr::Lit(Value::Str(w)) }
            TokenKind::Dollar       => { self.advance();
                let n = match self.advance().clone() { TokenKind::Word(w)=>w, _=>String::new() };
                Expr::Var(n)
            }
            _ => { self.advance(); Expr::Lit(Value::Null) }
        }
    }

    fn parse_pane(&mut self) -> PaneSpec {
        let mut split = SplitDir::Horizontal;
        let mut ratio = 50u8;
        let mut title = None;
        let mut border = BorderStyle::Rounded;
        // parse optional key=value attrs
        while matches!(self.peek(), TokenKind::Word(_)) {
            let key = match self.advance().clone() { TokenKind::Word(w)=>w, _=>break };
            if matches!(self.peek(), TokenKind::Equals) { self.advance(); }
            match key.as_str() {
                "split"  => { if let TokenKind::Word(v) = self.advance().clone() {
                    split = if v=="vertical"||v=="v" { SplitDir::Vertical } else { SplitDir::Horizontal };
                }}
                "ratio"  => { if let TokenKind::Number(n) = self.advance().clone() { ratio = n as u8; } }
                "title"  => { if let TokenKind::StringLit(s)|TokenKind::Word(s) = self.advance().clone() { title = Some(s); } }
                "border" => { if let TokenKind::Word(b) = self.advance().clone() {
                    border = match b.as_str() { "none"=>BorderStyle::None,"single"=>BorderStyle::Single,"double"=>BorderStyle::Double,"thick"=>BorderStyle::Thick, _=>BorderStyle::Rounded };
                }}
                _ => {}
            }
        }
        if matches!(self.peek(), TokenKind::LBrace) { self.advance(); }
        let children = self.parse_block_until(&[TokenKind::RBrace]);
        if matches!(self.peek(), TokenKind::RBrace) { self.advance(); }
        PaneSpec { split, ratio, children, border, title }
    }

    fn parse_tab(&mut self) -> TabSpec {
        let name = match self.advance().clone() {
            TokenKind::StringLit(s)|TokenKind::Word(s) => s,
            _ => String::from("tab"),
        };
        let icon = if let TokenKind::StringLit(s)|TokenKind::Word(s) = self.peek().clone() {
            let s = s.clone(); self.advance(); Some(s)
        } else { None };
        if matches!(self.peek(), TokenKind::LBrace) { self.advance(); }
        let children = self.parse_block_until(&[TokenKind::RBrace]);
        if matches!(self.peek(), TokenKind::RBrace) { self.advance(); }
        TabSpec { name, icon, children }
    }

    fn parse_widget(&mut self) -> WidgetKind {
        match self.advance().clone() {
            TokenKind::Word(w) => match w.as_str() {
                "clock"   => WidgetKind::Clock,
                "process" => WidgetKind::ProcessViewer,
                "editor"  => WidgetKind::Editor { file: None },
                "palette" => WidgetKind::CommandPalette,
                "status"  => WidgetKind::StatusBar,
                "filetree"|"files" => {
                    let path = if let TokenKind::StringLit(s)|TokenKind::Word(s) = self.peek().clone() {
                        let s=s.clone(); self.advance(); s
                    } else { String::from(".") };
                    WidgetKind::FileTree { path }
                }
                other => WidgetKind::Custom(String::from(other)),
            }
            _ => WidgetKind::StatusBar,
        }
    }

    fn parse_render(&mut self) -> Expr {
        let text = match self.advance().clone() {
            TokenKind::StringLit(s)|TokenKind::Word(s) => s,
            _ => String::new(),
        };
        let mut color = None;
        if matches!(self.peek(), TokenKind::Word(_)) {
            if let TokenKind::Word(k) = self.peek().clone() {
                if k == "color" { self.advance();
                    if matches!(self.peek(), TokenKind::Equals) { self.advance(); }
                    if let TokenKind::Word(c)|TokenKind::StringLit(c) = self.advance().clone() {
                        color = Some(c);
                    }
                }
            }
        }
        Expr::RenderExpr { text, color }
    }
}

// PartialEq: TokenKind derives PartialEq in lexer/mod.rs
