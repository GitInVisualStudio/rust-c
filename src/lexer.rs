use core::fmt;
use regex::Regex;
use std::{
    fs::File,
    io::{Error, Read},
};

pub mod tokens;

use tokens::{Token, TOKEN_PATTERN};

#[derive(Debug, Clone)]
pub struct LexerError {
    msg: String,
    line: usize,
    index: usize,
}

pub struct Lexer {
    content: String,
    index: usize,
    last_index: usize,
    patterns: Vec<(Token, Regex)>,
}

impl Lexer {
    pub fn error<T>(&self, string: String) -> Result<T, LexerError> {
        let mut line_breaks = 1;
        let mut index = 0;
        for (i, c) in self.content.bytes().enumerate() {
            if i == self.index {
                break;
            }
            index += 1;
            if c == b'\n' {
                index = 0;
                line_breaks += 1;
            }
        }
        Err(LexerError {
            msg: string,
            line: line_breaks,
            index: index,
        })
    }

    pub fn new(file_name: &str) -> Result<Lexer, Error> {
        let mut file = File::open(file_name)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let patterns: Vec<(Token, Regex)> = TOKEN_PATTERN
            .iter()
            .enumerate()
            .map(|(index, pattern)| (Token::from(index), Regex::new(pattern).unwrap()))
            .collect();
        Ok(Lexer {
            content: content,
            index: 0,
            last_index: 0,
            patterns: patterns,
        })
    }

    pub fn next(&mut self) -> Token {
        while self.index < self.content.len()
            && self.content.as_bytes()[self.index].is_ascii_whitespace()
        {
            self.index += 1;
        }
        if self.index == self.content.len() {
            self.last_index = self.index;
            return Token::EOF;
        }

        let result = self
            .patterns
            .iter()
            .map(|(token, regex)| (token, regex.find_at(&self.content, self.index)))
            .filter_map(|(token, x)| match x {
                Some(m) if m.start() == self.index && m.len() > 0 => Some((token, m.end())),
                Some(_) => None,
                None => None,
            })
            .next();
        if let Some((token, end)) = result {
            self.last_index = self.index;
            self.index = end;
            return token.clone();
        }
        Token::ERR
    }

    pub fn expect(&mut self, token: Token) -> Result<&str, LexerError> {
        let next_token = self.next();
        if next_token != token {
            return self.error(format!(
                "Unexpected token: {:?} expected: {:?}",
                next_token, token
            ));
        }
        Ok(self.last_string())
    }

    pub fn expect_tokens(&mut self, tokens: &[Token]) -> Result<&str, LexerError> {
        for token in tokens {
            let next_token = self.next();
            if next_token != *token {
                return self.error(format!(
                    "Unexpected token: {:?} expected: {:?}",
                    next_token, token
                ));
            }
        }
        Ok(self.last_string())
    }

    pub fn last_string(&self) -> &str {
        &self.content[self.last_index..self.index]
    }

    pub fn set_back(&mut self, index: usize) {
        self.index -= index;
    } 

    pub fn peek(&mut self) -> Token {
        let last_index = self.last_index;
        let result = self.next();
        // reset the index again
        self.index = self.last_index;
        self.last_index = last_index;
        result
    }

    pub fn peek_str(&mut self) -> &str {
        let last_index = self.last_index;
        self.next();
        let result = &self.content[self.last_index..self.index];
        // reset the index again
        self.index = self.last_index;
        self.last_index = last_index;
        result
    }
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error: '{}' at {}:{}", self.msg, self.line, self.index)
    }
}
