use core::fmt;
use regex::Regex;
pub mod tokens;

use tokens::{Token, TOKEN_PATTERN};

#[derive(Debug, Clone)]
pub struct LexerError {
    msg: String,
    line: usize,
    index: usize,
}

pub struct Lexer<'a> {
    index: usize,
    tokens: Vec<TokenView<'a>>,
}

#[derive(Clone, Copy)]
struct TokenView<'a> {
    token: Token,
    str: &'a str,
    line: usize,
    col: usize,
}

impl Lexer<'_> {
    pub fn error<T>(&self, string: String) -> Result<T, LexerError> {
        let last_token = self.tokens[self.index];
        Err(LexerError {
            msg: string,
            line: last_token.line,
            index: last_token.col,
        })
    }

    pub fn new(content: &str) -> Lexer {

        let patterns: Vec<(Token, Regex)> = TOKEN_PATTERN
            .iter()
            .enumerate()
            .map(|(index, pattern)| (Token::from(index), Regex::new(pattern).unwrap()))
            .collect();

        let mut tokens = Vec::new();
        let mut index = 0;
        let mut last_index;

        let mut column = 0;
        let mut line_breaks = 1;

        loop {
            last_index = index;
            let next_token = Self::next_token(&mut index, content, &patterns);
            let token_string = &content[last_index..index];

            for i in token_string.bytes() {
                column += 1;
                if i == b'\n' {
                    line_breaks += 1;
                    column = 0;
                }
            }
            
            tokens.push(TokenView {
                token: next_token,
                str: token_string.trim(),
                line: line_breaks,
                col: column,
            });

            if next_token == Token::EOF || next_token == Token::ERR {
                break;
            }
        }

        Lexer {
            index: 0,
            tokens: tokens,
        }
    }

    fn next_token(index: &mut usize, content: &str, patterns: &Vec<(Token, Regex)>) -> Token {
        while *index < content.len() && content.as_bytes()[*index].is_ascii_whitespace() {
            *index += 1;
        }
        if *index == content.len() {
            return Token::EOF;
        }

        let result = patterns
            .iter()
            .map(|(token, regex)| (token, regex.find_at(&content, *index)))
            .filter_map(|(token, x)| match x {
                Some(m) if m.start() == *index && m.len() > 0 => Some((token, m.end())),
                Some(_) => None,
                None => None,
            })
            .next();
        if let Some((token, end)) = result {
            *index = end;
            return token.clone();
        }
        Token::ERR
    }

    #[inline]
    pub fn next(&mut self) -> Token {
        if self.index >= self.tokens.len() {
            return Token::EOF;
        }
        let token = self.tokens[self.index].token;
        self.index += 1;
        token
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

    #[inline]
    pub fn last_string(&self) -> &str {
        self.tokens[self.index - 1].str
    }

    #[inline]
    pub fn peek(&mut self) -> Token {
        self.tokens[self.index].token
    }

    #[inline]
    pub fn peek_str(&mut self) -> &str {
        self.tokens[self.index].str
    }
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error: '{}' at {}:{}", self.msg, self.line, self.index)
    }
}
