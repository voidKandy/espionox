use super::errors::{LexerError, LexerResult};
use super::tokens::*;
use std::char;
use tracing::{info, warn};

#[derive(Debug)]
pub struct Lexer {
    input: String,
    position: usize,      // current position in input (points to current char)
    read_position: usize, // current reading position in input (after current char)
    ch: Option<char>,     // NONE if at end of input
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.to_lowercase().to_owned(),
            position: 0,
            read_position: 1,
            ch: input.to_lowercase().chars().nth(0),
        }
    }

    #[tracing::instrument(name = "read char")]
    fn read_char(&mut self) {
        self.ch = self.input.chars().nth(self.read_position);
        info!("char set to: {:?}", self.ch);
        self.position = self.read_position;
        self.read_position += 1
    }

    fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.read_position)
    }

    #[tracing::instrument(name = "lex input")]
    pub fn lex_input(&mut self) -> LexerResult<Vec<Token>> {
        let mut output = vec![];
        while let Some(c) = self.ch {
            info!("current char: {}", c);
            match c {
                '(' => output.push(Token::try_from(LP)?),
                ')' => output.push(Token::try_from(RP)?),
                ':' => output.push(Token::try_from(COLON)?),
                ',' => output.push(Token::try_from(COMMA)?),
                '!' => output.push(Token::try_from(BANG)?),
                '|' => output.push(Token::try_from(PIPE)?),
                '\'' => {
                    info!("In Token::StrLiteral branch");
                    let mut str_lit = String::new();
                    let mut next = self.peek().unwrap_or(0 as char);
                    while next != '\'' {
                        str_lit.push(next);
                        self.read_char();
                        next = self.peek().unwrap_or(0 as char);
                    }
                    self.read_char();
                    warn!("Pushing Token::StrLiteral: {}", str_lit);
                    output.push(Token::new_str_literal(&str_lit))
                }

                alph if c.is_alphabetic() || c == '_' => {
                    info!("In Token::Identifier branch");
                    let mut literal = String::from(alph);
                    let mut next = self.peek().unwrap_or(0 as char);
                    while next.is_alphabetic() || next == '_' {
                        literal.push(next);
                        self.read_char();
                        next = self.peek().unwrap_or(0 as char);
                    }
                    match Token::try_from(literal.as_str()).ok() {
                        Some(tok) => output.push(tok),
                        None => {
                            warn!("Pushing Token::Identifier: {}", literal);
                            output.push(Token::new_identifier(&literal));
                        }
                    }
                }

                other => {
                    if other != ' ' {
                        warn!("encountered other non-whitespace token: {}", other);
                    }
                }
            }
            info!("restarting loop\n {:?}", output);
            self.read_char();
        }
        output.push(Token::end());
        Ok(output)
    }
}
