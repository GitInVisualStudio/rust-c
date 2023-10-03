use crate::{error::Error, lexer::tokens::TokenKind, parser::Parser, visitor::Visitable};

use super::expression::Expression;

#[derive(Debug)]
pub struct FunctionCall<'a> {
    pub(crate) name: &'a str,
    pub(crate) parameter: Vec<Expression<'a>>,
}

impl Visitable for FunctionCall<'_> {}

impl<'a> Parser<'a> {
    pub fn function_call(&mut self) -> Result<FunctionCall<'a>, Error<'a>> {
        let name = self.last_token().1.string();

        let mut parameter: Vec<Expression> = Vec::new();
        self.expect(TokenKind::LPAREN)?;

        while self.peek() != TokenKind::RPAREN {
            parameter.push(self.expression()?);
            if self.peek() == TokenKind::COMMA {
                self.next();
            }
        }
        self.next();

        Ok(FunctionCall {
            name: name,
            parameter: parameter,
        })
    }
}
