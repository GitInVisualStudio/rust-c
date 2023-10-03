use derive_getters::Getters;

use crate::{error::Error, lexer::tokens::TokenKind, parser::Parser};

use super::{statement::Statement, ASTNode};

#[derive(Debug, Getters)]
pub struct Compound<'a> {
    statements: Vec<Statement<'a>>,
}

impl ASTNode for Compound<'_> {}

impl<'a> Parser<'a> {
    pub fn compound_statement(&mut self) -> Result<Compound<'a>, Error<'a>> {
        if self.peek() != TokenKind::LCURL {
            let statements = vec![self.statement()?];
            return Ok(Compound {
                statements: statements,
            });
        }
        self.next();
        let mut statements = Vec::new();
        while self.peek() != TokenKind::RCURL {
            let statement = self.statement()?;
            statements.push(statement);
        }
        self.next();
        Ok(Compound {
            statements: statements,
        })
    }
}
