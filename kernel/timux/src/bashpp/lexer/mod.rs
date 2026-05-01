//! Bash++ Lexer — tokenizes both Bash++ and bash-compatible syntax
extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Word(String), StringLit(String), RawString(String), Number(i64), Bool(bool),
    Arrow, FatArrow, Walrus, DoublePipe, DoubleAmp, Pipe, Semicolon, Newline,
    Amp, Bang, Dollar, DollarLBrace, DollarLParen,
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Equals, PlusEquals, Lt, Gt, GtGt, LtLt, LtAmp, GtAmp,
    At, Hash, Tilde, Star, Question, Dot, DotDot, Comma, Colon,
    // Bash++ keywords
    KwLet, KwFn, KwReturn, KwIf, KwElse, KwFor, KwWhile, KwLoop,
    KwBreak, KwContinue, KwMatch, KwIn,
    // Bash compat keywords
    KwDo, KwDone, KwThen, KwFi, KwCase, KwEsac,
    KwExport, KwUnset, KwSource, KwExec, KwEval, KwExit,
    // TUI keywords
    KwPane, KwTab, KwWidget, KwRender, KwIcon, KwBorder, KwTheme,
    Eof, Unknown(char),
}

#[derive(Debug, Clone)]
pub struct Token { pub kind: TokenKind, pub line: u32, pub col: u32 }

impl Token {
    pub fn new(kind: TokenKind, line: u32, col: u32) -> Self { Self { kind, line, col } }
}

pub struct Lexer { source: Vec<char>, pos: usize, line: u32, col: u32 }

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self { source: source.chars().collect(), pos: 0, line: 1, col: 1 }
    }

    fn peek(&self) -> Option<char> { self.source.get(self.pos).copied() }
    fn peek2(&self) -> Option<char> { self.source.get(self.pos + 1).copied() }

    fn advance(&mut self) -> Option<char> {
        let c = self.source.get(self.pos).copied()?;
        self.pos += 1;
        if c == '\n' { self.line += 1; self.col = 1; } else { self.col += 1; }
        Some(c)
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(' ') | Some('\t') | Some('\r')) { self.advance(); }
    }

    fn read_word(&mut self, first: char) -> String {
        let mut s = String::new(); s.push(first);
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || matches!(c, '_'|'-'|'.'|'/'|':') { s.push(c); self.advance(); }
            else { break; }
        }
        s
    }

    fn read_string(&mut self, q: char) -> String {
        let mut s = String::new();
        while let Some(c) = self.advance() {
            if c == q { break; }
            if c == '\\' {
                if let Some(e) = self.advance() {
                    match e { 'n'=>s.push('\n'), 't'=>s.push('\t'), o=>{s.push('\\');s.push(o);} }
                }
            } else { s.push(c); }
        }
        s
    }

    fn read_num(&mut self, first: char) -> i64 {
        let mut s = String::new(); s.push(first);
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) { s.push(self.advance().unwrap()); }
        s.parse().unwrap_or(0)
    }

    fn kw(w: &str) -> TokenKind {
        match w {
            "let"=>TokenKind::KwLet,"fn"=>TokenKind::KwFn,"return"=>TokenKind::KwReturn,
            "if"=>TokenKind::KwIf,"else"=>TokenKind::KwElse,"for"=>TokenKind::KwFor,
            "while"=>TokenKind::KwWhile,"loop"=>TokenKind::KwLoop,"break"=>TokenKind::KwBreak,
            "continue"=>TokenKind::KwContinue,"match"=>TokenKind::KwMatch,"in"=>TokenKind::KwIn,
            "do"=>TokenKind::KwDo,"done"=>TokenKind::KwDone,"then"=>TokenKind::KwThen,
            "fi"=>TokenKind::KwFi,"case"=>TokenKind::KwCase,"esac"=>TokenKind::KwEsac,
            "export"=>TokenKind::KwExport,"unset"=>TokenKind::KwUnset,"source"=>TokenKind::KwSource,
            "exec"=>TokenKind::KwExec,"eval"=>TokenKind::KwEval,"exit"=>TokenKind::KwExit,
            "pane"=>TokenKind::KwPane,"tab"=>TokenKind::KwTab,"widget"=>TokenKind::KwWidget,
            "render"=>TokenKind::KwRender,"icon"=>TokenKind::KwIcon,"border"=>TokenKind::KwBorder,
            "theme"=>TokenKind::KwTheme,"true"=>TokenKind::Bool(true),"false"=>TokenKind::Bool(false),
            o=>TokenKind::Word(String::from(o)),
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut out = Vec::new();
        loop {
            self.skip_ws();
            let (l,c) = (self.line, self.col);
            let ch = match self.advance() {
                None => { out.push(Token::new(TokenKind::Eof,l,c)); break; }
                Some(x) => x,
            };
            let kind = match ch {
                '\n' => TokenKind::Newline,
                '#'  => { while matches!(self.peek(), Some(x) if x != '\n') { self.advance(); } continue; }
                '"'  => TokenKind::StringLit(self.read_string('"')),
                '\'' => TokenKind::RawString(self.read_string('\'')),
                '|'  => if self.peek()==Some('|'){self.advance();TokenKind::DoublePipe}else{TokenKind::Pipe},
                '&'  => if self.peek()==Some('&'){self.advance();TokenKind::DoubleAmp}else{TokenKind::Amp},
                '-'  => if self.peek()==Some('>'){self.advance();TokenKind::Arrow}else{TokenKind::Word(String::from("-"))},
                '='  => if self.peek()==Some('>'){self.advance();TokenKind::FatArrow}else{TokenKind::Equals},
                ':'  => if self.peek()==Some('='){self.advance();TokenKind::Walrus}else{TokenKind::Colon},
                '+'  => if self.peek()==Some('='){self.advance();TokenKind::PlusEquals}else{TokenKind::Word(String::from("+"))},
                '<'  => match self.peek(){Some('<')=>{self.advance();TokenKind::LtLt},Some('&')=>{self.advance();TokenKind::LtAmp},_=>TokenKind::Lt},
                '>'  => match self.peek(){Some('>')=>{self.advance();TokenKind::GtGt},Some('&')=>{self.advance();TokenKind::GtAmp},_=>TokenKind::Gt},
                '$'  => match self.peek(){Some('{')=>{self.advance();TokenKind::DollarLBrace},Some('(')=>{self.advance();TokenKind::DollarLParen},_=>TokenKind::Dollar},
                '.'  => if self.peek()==Some('.'){self.advance();TokenKind::DotDot}else{TokenKind::Dot},
                ';'=>TokenKind::Semicolon,'!'=>TokenKind::Bang,'('=>TokenKind::LParen,')'=>TokenKind::RParen,
                '{'=>TokenKind::LBrace,'}'=>TokenKind::RBrace,'['=>TokenKind::LBracket,']'=>TokenKind::RBracket,
                '~'=>TokenKind::Tilde,'*'=>TokenKind::Star,'?'=>TokenKind::Question,
                '@'=>TokenKind::At,','=>TokenKind::Comma,
                c if c.is_ascii_digit() => TokenKind::Number(self.read_num(c)),
                c if c.is_alphabetic()||c=='_' => { let w=self.read_word(c); Self::kw(&w) }
                o => TokenKind::Unknown(o),
            };
            out.push(Token::new(kind,l,c));
        }
        out
    }
}
