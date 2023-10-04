use crate::error::Error;
use crate::lexer::tokens::TokenKind;
use crate::parser::Parser;
use crate::visitor::Visitable;

use super::function::Function;
use super::statement::Statement;

#[derive(Debug)]
pub enum Decalrations<'a> {
    Statement(&'a Statement<'a>),
    Function(&'a Function<'a>),
}

#[derive(Debug)]
pub struct Program<'a> {
    pub(crate) declarations: Vec<Decalrations<'a>>,
}

impl Visitable for Program<'_> {}

impl<'a> Parser<'a> {
    pub fn program(&mut self) -> Result<&'a Program<'a>, Error<'a>> {
        let mut declarations: Vec<Decalrations> = Vec::new();
        while self.peek() != TokenKind::EOF {
            declarations.push(match self.peek() {
                TokenKind::TYPEDEF | TokenKind::STRUCT => {
                    Decalrations::Statement(self.statement()?)
                }
                _ => Decalrations::Function(self.function()?),
            })
        }
        self.expect(TokenKind::EOF)?;
        Ok(self.alloc(Program {
            declarations: declarations,
        }))
    }
}
