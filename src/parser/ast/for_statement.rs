use crate::{error::Error, lexer::tokens::TokenKind, parser::Parser, visitor::Visitable};

use super::{compound_statement::Compound, expression::Expression, statement::Statement};

#[derive(Debug)]
pub struct ForStatement<'a> {
    pub(crate) init: &'a Statement<'a>,
    pub(crate) condition: &'a Expression<'a>,
    pub(crate) post: Option<&'a Expression<'a>>,
    pub(crate) body: &'a Compound<'a>,
}

impl Visitable for ForStatement<'_> {}

impl<'a> Parser<'a> {
    pub fn for_statement(&mut self) -> Result<&'a ForStatement<'a>, Error<'a>> {
        self.expect(TokenKind::FOR)?;
        self.expect(TokenKind::LPAREN)?;
        let init = self.bump.alloc(self.statement()?);

        let condition;
        if self.peek() != TokenKind::SEMIC {
            condition = self.expression()?;
        } else {
            condition = self.alloc(Expression::IntLiteral(1));
        }
        self.expect(TokenKind::SEMIC)?;

        let post;
        if self.peek() != TokenKind::RPAREN {
            post = Some(self.expression()?);
        } else {
            post = None;
        }
        self.expect(TokenKind::RPAREN)?;

        let body = self.compound_statement()?;

        Ok(self.alloc(ForStatement {
            init,
            condition,
            post,
            body,
        }))
    }
}
