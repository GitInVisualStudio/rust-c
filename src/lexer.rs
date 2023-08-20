use std::{fs::File, io::{Error,  Read}};
use regex::{Regex};

pub mod tokens;

use tokens::{Token, TOKEN_PATTERN};

pub struct Lexer {
    content: String,
    index: usize,
    patterns: Vec<(Token, Regex)>
}

impl Lexer {
    pub fn new(file_path: &str) -> Result<Lexer, Error> {
        let mut file = File::open(file_path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let patterns: Vec<(Token, Regex)> = TOKEN_PATTERN
            .iter()
            .enumerate()
            .map(|(index, pattern)| {
                (Token::from(index), Regex::new(pattern).unwrap())
            }).collect();
        Ok(Lexer {content: content, index: 0, patterns: patterns})
        
    }

    pub fn next(&mut self) -> Token {
        while self.index < self.content.len() && self.content.as_bytes()[self.index].is_ascii_whitespace() {
            self.index += 1;
        }
        if self.index == self.content.len() {
            return Token::EOF;
        }
        let (first_token, first_match) = self.patterns
            .iter()
            .map(|(token, regex)| (token, regex.find_at(&self.content, self.index)))
            .filter(|(_, m)| m.is_some())
            .map(|(token, m)| (token, m.unwrap()))
            .min_by(|(_, a), (_, b)| a.start().cmp(&b.start())).unwrap();
        if first_match.start() != self.index {
            return Token::ERR;
        }
        self.index = first_match.end();
        first_token.clone()
    }
}