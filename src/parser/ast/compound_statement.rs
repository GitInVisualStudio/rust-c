use derive_getters::Getters;

use crate::{error::Error, lexer::tokens::TokenKind, parser::Parser, visitor::Visitable};

use super::statement::Statement;

#[derive(Debug)]
pub struct Compound<'a> {
    pub(crate) statements: Vec<Statement<'a>>,
}

impl Visitable for Compound<'_> {}

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
