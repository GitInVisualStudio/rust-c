use bumpalo::Bump;

use crate::{
    error::Error,
    lexer::{tokens::TokenKind, SrcLocation, Token},
};

use self::ast::expression::Expression;

pub mod scope;
pub mod ast;

pub struct Parser<'a> {
    pub(crate) bump: &'a Bump,
    pub(crate) assignee: Option<&'a Expression<'a>>,
    tokens: &'a [Token<'a>],
    index: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token<'a>], bump: &'a Bump) -> Self {
        Parser {
            tokens,
            bump,
            index: 0,
            assignee: None,
        }
    }

    pub fn next_kind(&mut self) -> TokenKind {
        self.next().0
    }

    pub fn next(&mut self) -> Token<'a> {
        if self.index >= self.tokens.len() {
            return *self.tokens.last().unwrap();
        }
        let token = self.tokens[self.index];
        self.index += 1;
        token
    }

    pub fn expect(&mut self, token: TokenKind) -> Result<SrcLocation<'a>, Error<'a>> {
        let (next_token, location) = self.next();
        if next_token != token {
            return Err(Error::UnexpectedToken {
                expected: token,
                found: next_token,
                location: location,
            });
        }
        Ok(location)
    }

    pub fn peek(&self) -> TokenKind {
        self.tokens[self.index].0
    }

    pub fn ahead(&self, count: usize) -> TokenKind {
        self.tokens[self.index + count].0
    }

    pub fn last_token(&self) -> Token<'a> {
        self.tokens[self.index - 1]
    }

    pub fn current(&self) -> Token<'a> {
        self.tokens[self.index]
    }
}
