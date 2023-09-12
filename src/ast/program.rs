use std::rc::Rc;

use derive_getters::Getters;

use super::ASTNode;
use super::function::Function;
use super::statement::Statement;
use crate::lexer::tokens::Token;
use crate::lexer::LexerError;
use crate::parser::{Parse, Parser};

#[derive(Debug)]
pub enum Decalrations {
    Statement(Statement),
    Function(Rc<Function>),
}

#[derive(Debug, Getters)]
pub struct Program {
    declarations: Vec<Decalrations>,
}

impl ASTNode for Program {}

impl Parse<Program> for Parser<'_> {
    fn parse(&mut self) -> Result<Program, LexerError> {
        self.scope.push();
        let mut declarations: Vec<Decalrations> = Vec::new();
        while self.peek() != Token::EOF {
            declarations.push(match self.peek() {
                Token::TYPEDEF | Token::STRUCT => Decalrations::Statement(self.parse()?),
                _ => Decalrations::Function(self.parse()?),
            })
        }
        self.expect(Token::EOF)?;
        Ok(Program {
            declarations: declarations,
        })
    }
}
