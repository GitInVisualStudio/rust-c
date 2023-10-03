use crate::{error::Error, lexer::tokens::TokenKind, parser::Parser};

use super::{compound_statement::Compound, expression::Expression, ASTNode};

#[derive(Debug)]
pub struct WhileStatement<'a> {
    condition: Expression<'a>,
    body: Compound<'a>,
}

impl ASTNode for WhileStatement<'_> {}

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
