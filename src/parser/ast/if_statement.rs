use crate::{error::Error, lexer::tokens::TokenKind, parser::Parser, visitor::Visitable};

use super::{compound_statement::Compound, expression::Expression};

#[derive(Debug)]
pub enum ElsePart<'a> {
    IfStatement(&'a IfStatement<'a>),
    Compound(Compound<'a>),
    None,
}

#[derive(Debug)]
pub struct IfStatement<'a> {
    pub(crate) statements: Compound<'a>,
    pub(crate) condition: Expression<'a>,
    pub(crate) else_part: ElsePart<'a>,
}

impl Visitable for IfStatement<'_> {}

impl<'a> Parser<'a> {
    pub fn if_statement(&mut self) -> Result<IfStatement<'a>, Error<'a>> {
        self.expect(TokenKind::IF)?;
        self.expect(TokenKind::LPAREN)?;
        let condition = self.expression()?;
        self.expect(TokenKind::RPAREN)?;
        let statements = self.compound_statement()?;
        let mut else_part: ElsePart = ElsePart::None;
        if self.peek() == TokenKind::ELSE {
            self.next();
            else_part = match self.peek() {
                TokenKind::IF => ElsePart::IfStatement(self.bump.alloc(self.if_statement()?)),
                _ => ElsePart::Compound(self.compound_statement()?),
            };
        }
        Ok(IfStatement {
            statements: statements,
            condition: condition,
            else_part: else_part,
        })
    }
}
