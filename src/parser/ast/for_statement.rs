use crate::{error::Error, lexer::tokens::TokenKind, parser::Parser};

use super::{compound_statement::Compound, expression::Expression, statement::Statement, ASTNode};

#[derive(Debug)]
pub struct ForStatement<'a> {
    init: &'a Statement<'a>,
    condition: Expression<'a>,
    post: Option<Expression<'a>>,
    body: Compound<'a>,
}

impl ASTNode for ForStatement<'_> {}

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
