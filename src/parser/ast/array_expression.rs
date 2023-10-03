use crate::{error::Error, lexer::tokens::TokenKind, parser::Parser};

use super::{ASTNode, expression::Expression};

#[derive(Debug)]
pub enum ArrayExpression<'a> {
    StackArray { expressinos: Vec<Expression<'a>> },
    StringLiteral { string: &'a str },
}

impl ASTNode for ArrayExpression<'_> {}

impl<'a> Parser<'a> {
    pub fn array_expression(&mut self) -> Result<ArrayExpression<'a>, Error<'a>> {
        Ok(match self.peek() {
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
                ArrayExpression::StackArray {
                    expressinos: expressions,
                }
            }
        })
    }
}
