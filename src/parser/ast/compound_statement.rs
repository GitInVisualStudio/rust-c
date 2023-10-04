use crate::{error::Error, lexer::tokens::TokenKind, parser::Parser, visitor::Visitable};

use super::statement::Statement;

#[derive(Debug)]
pub struct Compound<'a> {
    pub(crate) statements: Vec<&'a Statement<'a>>,
}

impl Visitable for Compound<'_> {}

impl<'a> Parser<'a> {
    pub fn compound_statement(&mut self) -> Result<&'a Compound<'a>, Error<'a>> {
        if self.peek() != TokenKind::LCURL {
            let statements = vec![self.statement()?];
            return Ok(self.alloc(Compound {
                statements: statements,
            }));
        }
        self.next();
        let mut statements = Vec::new();
        while self.peek() != TokenKind::RCURL {
            let statement = self.statement()?;
            statements.push(statement);
        }
        self.next();
        Ok(self.alloc(Compound {
            statements: statements,
        }))
    }
}
