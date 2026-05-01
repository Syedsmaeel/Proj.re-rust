//! Bash++ Lite Lexer for TBM
//!
//! Supports a subset of Bash++ for bootloader configuration.

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Set,
    Entry,
    Widget,
    Theme,
    Identifier(re_core::protocol::StaticStr),
    String(re_core::protocol::StaticStr),
    Equals,
    OpenBrace,
    CloseBrace,
    EOF,
}

pub struct Lexer {
    input: &'static str,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &'static str) -> Self {
        Self { input, pos: 0 }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if self.pos >= self.input.len() {
            return Token::EOF;
        }

        let ch = self.input.as_bytes()[self.pos];

        match ch {
            b'=' => { self.pos += 1; Token::Equals }
            b'{' => { self.pos += 1; Token::OpenBrace }
            b'}' => { self.pos += 1; Token::CloseBrace }
            b'"' => self.read_string(),
            _ if ch.is_ascii_alphabetic() => self.read_identifier(),
            _ => { self.pos += 1; self.next_token() } // Skip unknown
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn read_identifier(&mut self) -> Token {
        let start = self.pos;
        while self.pos < self.input.len() && (self.input.as_bytes()[self.pos].is_ascii_alphanumeric() || self.input.as_bytes()[self.pos] == b'_') {
            self.pos += 1;
        }
        let id = &self.input[start..self.pos];
        match id {
            "set" => Token::Set,
            "entry" => Token::Entry,
            "widget" => Token::Widget,
            "theme" => Token::Theme,
            _ => Token::Identifier(re_core::protocol::StaticStr::new(id)),
        }
    }

    fn read_string(&mut self) -> Token {
        self.pos += 1; // skip "
        let start = self.pos;
        while self.pos < self.input.len() && self.input.as_bytes()[self.pos] != b'"' {
            self.pos += 1;
        }
        let s = &self.input[start..self.pos];
        self.pos += 1; // skip "
        Token::String(re_core::protocol::StaticStr::new(s))
    }
}
