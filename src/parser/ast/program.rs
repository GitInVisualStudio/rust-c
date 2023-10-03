use derive_getters::Getters;

use crate::error::Error;
use crate::lexer::tokens::TokenKind;
use crate::parser::Parser;
use crate::visitor::Visitable;

use super::function::Function;
use super::statement::Statement;

#[derive(Debug)]
pub enum Decalrations<'a> {
    Statement(Statement<'a>),
    Function(&'a Function<'a>),
}

#[derive(Debug, Getters)]
pub struct Program<'a> {
    pub(crate) declarations: Vec<Decalrations<'a>>,
}

impl Visitable for Program<'_> {}

impl<'a> Parser<'a> {
    pub fn program(&mut self) -> Result<Program<'a>, Error<'a>> {
        let mut declarations: Vec<Decalrations> = Vec::new();
        while self.peek() != TokenKind::EOF {
            declarations.push(match self.peek() {
                TokenKind::TYPEDEF | TokenKind::STRUCT => {
                    Decalrations::Statement(self.statement()?)
                }
                _ => Decalrations::Function(self.bump.alloc(self.function()?)),
            })
        }
        self.expect(TokenKind::EOF)?;
        Ok(Program {
            declarations: declarations,
        })
    }
}
