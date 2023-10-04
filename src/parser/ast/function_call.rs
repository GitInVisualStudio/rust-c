use crate::{error::Error, lexer::tokens::TokenKind, parser::Parser, visitor::Visitable};

use super::expression::Expression;

#[derive(Debug)]
pub struct FunctionCall<'a> {
    pub(crate) name: &'a str,
    pub(crate) parameter: Vec<&'a Expression<'a>>,
}

impl Visitable for FunctionCall<'_> {}

impl<'a> Parser<'a> {
    pub fn function_call(&mut self) -> Result<&'a FunctionCall<'a>, Error<'a>> {
        let name = self.last_token().1.string();

        self.expect(TokenKind::LPAREN)?;

        let mut parameter = Vec::new();
        while self.peek() != TokenKind::RPAREN {
            parameter.push(self.expression()?);
            if self.peek() == TokenKind::COMMA {
                self.next();
            }
        }
        self.next();

        Ok(self.alloc(FunctionCall {
            name: name,
            parameter: parameter,
        }))
    }
}
