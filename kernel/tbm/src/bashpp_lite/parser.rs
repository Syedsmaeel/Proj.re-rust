//! Bash++ Lite Parser for TBM

use crate::bashpp_lite::lexer::{Lexer, Token};
use re_core::protocol::StaticStr;

#[derive(Debug)]
pub enum Command {
    SetTheme(StaticStr),
    AddEntry { name: StaticStr, profile: StaticStr },
    AddWidget { kind: StaticStr, pos: StaticStr },
}

pub struct Parser {
    lexer: Lexer,
    curr_token: Token,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let curr_token = lexer.next_token();
        Self { lexer, curr_token }
    }

    fn advance(&mut self) {
        self.curr_token = self.lexer.next_token();
    }

    pub fn parse_all(&mut self) -> [Option<Command>; 10] {
        let mut cmds: [Option<Command>; 10] = Default::default();
        let mut i = 0;

        while self.curr_token != Token::EOF && i < 10 {
            if let Some(cmd) = self.parse_command() {
                cmds[i] = Some(cmd);
                i += 1;
            }
            self.advance();
        }
        cmds
    }

    fn parse_command(&mut self) -> Option<Command> {
        match self.curr_token {
            Token::Set => {
                self.advance();
                if let Token::Identifier(id) = self.curr_token {
                    if id.as_str() == "theme" {
                        self.advance();
                        if self.curr_token == Token::Equals {
                            self.advance();
                            if let Token::String(val) = self.curr_token {
                                return Some(Command::SetTheme(val));
                            }
                        }
                    }
                }
                None
            }
            Token::Entry => {
                self.advance();
                if let Token::String(name) = self.curr_token {
                    self.advance();
                    if self.curr_token == Token::OpenBrace {
                        self.advance();
                        // Simplified: expect profile="something"
                        if let Token::Identifier(id) = self.curr_token {
                            if id.as_str() == "profile" {
                                self.advance();
                                if self.curr_token == Token::Equals {
                                    self.advance();
                                    if let Token::String(profile) = self.curr_token {
                                        return Some(Command::AddEntry { name, profile });
                                    }
                                }
                            }
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }
}
