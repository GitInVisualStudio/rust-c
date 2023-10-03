use crate::{error::Error, lexer::tokens::TokenKind, parser::Parser, visitor::Visitable};

use super::expression::Expression;

#[derive(Debug)]
pub struct StructExpression<'a> {
    pub(crate) fields: Vec<(&'a str, Expression<'a>)>,
}

impl Visitable for StructExpression<'_> {}

impl<'a> Parser<'a> {
    pub fn struct_expression(&mut self) -> Result<StructExpression<'a>, Error<'a>> {
        let mut fields = Vec::new();
        while self.peek() != TokenKind::RCURL {
            self.expect(TokenKind::DOT)?;
            let name = self.expect(TokenKind::IDENT)?.string();
            self.expect(TokenKind::ASSIGN)?;
            let expression = self.expression()?;
            fields.push((name, expression));
            if self.peek() != TokenKind::RCURL {
                self.expect(TokenKind::COMMA)?;
            }
        }

        self.expect(TokenKind::RCURL)?;
        return Ok(StructExpression { fields });
    }
}
