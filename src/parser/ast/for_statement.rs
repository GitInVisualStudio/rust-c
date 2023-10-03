use crate::{error::Error, lexer::tokens::TokenKind, parser::Parser, visitor::Visitable};

use super::{compound_statement::Compound, expression::Expression, statement::Statement};

#[derive(Debug)]
pub struct ForStatement<'a> {
    pub(crate) init: &'a Statement<'a>,
    pub(crate) condition: Expression<'a>,
    pub(crate) post: Option<Expression<'a>>,
    pub(crate) body: Compound<'a>,
}

impl Visitable for ForStatement<'_> {}

impl<'a> Parser<'a> {
    pub fn for_statement(&mut self) -> Result<ForStatement<'a>, Error<'a>> {
        self.expect(TokenKind::FOR)?;
        self.expect(TokenKind::LPAREN)?;
        let init = self.bump.alloc(self.statement()?);

        let condition;
        if self.peek() != TokenKind::SEMIC {
            condition = self.expression()?;
        } else {
            condition = Expression::IntLiteral(1);
        }
        self.expect(TokenKind::SEMIC)?;

        let post: Option<Expression>;
        if self.peek() != TokenKind::RPAREN {
            post = Some(self.expression()?);
        } else {
            post = None;
        }
        self.expect(TokenKind::RPAREN)?;

        let body = self.compound_statement()?;

        Ok(ForStatement {
            init,
            condition,
            post,
            body,
        })
    }
}
