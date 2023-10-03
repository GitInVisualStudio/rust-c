use crate::{error::Error, lexer::tokens::TokenKind, parser::Parser};

use super::{expression::Expression, ASTNode};

#[derive(Debug)]
pub enum TypeExpression<'a> {
    Primitive(TokenKind),
    Typeof(&'a Expression<'a>),
    Unresolved(&'a str),
    Struct {
        name: &'a str,
        fields: Vec<(&'a str, TypeExpression<'a>)>,
    },
    UnresolvedStruct(&'a str),
    Pointer(&'a TypeExpression<'a>),
}

impl ASTNode for TypeExpression<'_> {}

impl<'a> Parser<'a> {
    pub fn type_expression(&mut self) -> Result<TypeExpression<'a>, Error<'a>> {
        let mut type_expression = match self.next() {
            (TokenKind::STRUCT, _) => {
                let name = self.expect(TokenKind::IDENT)?.string();
                match self.peek() {
                    TokenKind::LCURL => {
                        self.expect(TokenKind::LCURL)?;
                        let mut fields = Vec::new();
                        while self.peek() != TokenKind::RCURL {
                            let type_expression = self.type_expression()?;
                            let name = self.expect(TokenKind::IDENT)?.string();
                            fields.push((name, type_expression));
                            self.expect(TokenKind::SEMIC)?;
                        }
                        self.next();

                        TypeExpression::Struct {
                            name: name,
                            fields: fields,
                        }
                    }
                    TokenKind::IDENT => TypeExpression::UnresolvedStruct(name),
                    _ => TypeExpression::UnresolvedStruct(name),
                }
            }
            (TokenKind::IDENT, location) => TypeExpression::Unresolved(location.src),
            (TokenKind::TYPEOF, _) => {
                self.expect(TokenKind::LPAREN)?;
                let expression: Expression = self.expression()?;
                self.expect(TokenKind::RPAREN)?;
                TypeExpression::Typeof(self.bump.alloc(expression))
            }
            (x, _) => TypeExpression::Primitive(x),
        };

        while self.peek() == TokenKind::MUL {
            self.next();
            let allocation = &*self.bump.alloc(type_expression);
            type_expression = TypeExpression::Pointer(allocation);
        }
        Ok(type_expression)
    }
}
