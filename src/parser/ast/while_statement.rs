use crate::{error::Error, lexer::tokens::TokenKind, parser::Parser, visitor::Visitable};

use super::{compound_statement::Compound, expression::Expression};

#[derive(Debug)]
pub struct WhileStatement<'a> {
    pub(crate) condition: Expression<'a>,
    pub(crate) body: Compound<'a>,
}

impl Visitable for WhileStatement<'_> {}

impl<'a> Parser<'a> {
    pub fn while_statement(&mut self) -> Result<WhileStatement<'a>, Error<'a>> {
        self.expect(TokenKind::WHILE)?;
        self.expect(TokenKind::LPAREN)?;
        let condition = self.expression()?;
        self.expect(TokenKind::RPAREN)?;
        let body = self.compound_statement()?;
        Ok(WhileStatement { condition, body })
    }
}
