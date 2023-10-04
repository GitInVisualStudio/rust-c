use crate::{error::Error, lexer::tokens::TokenKind, parser::Parser, visitor::Visitable};

use super::expression::Expression;

#[derive(Debug)]
pub enum ArrayExpression<'a> {
    StackArray {
        expressions: Vec<&'a Expression<'a>>,
    },
    StringLiteral {
        string: &'a str,
    },
}

impl Visitable for ArrayExpression<'_> {}

impl<'a> Parser<'a> {
    pub fn array_expression(&mut self) -> Result<&'a ArrayExpression<'a>, Error<'a>> {
        Ok(self.bump.alloc(match self.peek() {
            TokenKind::STRINGLIT => {
                let string = self.expect(TokenKind::STRINGLIT)?.string();
                ArrayExpression::StringLiteral { string }
            }
            _ => {
                let mut expressions = Vec::new();
                while self.peek() != TokenKind::RCURL {
                    let expr = self.expression()?;
                    expressions.push(expr);
                    if self.peek() == TokenKind::COMMA {
                        self.next();
                    }
                }
                self.next();
                ArrayExpression::StackArray { expressions }
            }
        }))
    }
}
