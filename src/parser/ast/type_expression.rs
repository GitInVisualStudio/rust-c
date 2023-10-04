use crate::{error::Error, lexer::tokens::TokenKind, parser::Parser, visitor::Visitable};

use super::expression::Expression;

#[derive(Debug)]
pub enum TypeExpression<'a> {
    Primitive(TokenKind),
    Typeof(&'a Expression<'a>),
    Named(&'a str),
    Struct {
        name: &'a str,
        fields: Vec<(&'a str, &'a TypeExpression<'a>)>,
    },
    NamedStruct(&'a str),
    Pointer(&'a TypeExpression<'a>),
}

impl<'a> Visitable for TypeExpression<'a> {}

impl<'a> Parser<'a> {
    pub fn type_expression(&mut self) -> Result<&'a TypeExpression<'a>, Error<'a>> {
        let mut type_expression = match self.next() {
            (TokenKind::STRUCT, _) => {
                let name = self.expect(TokenKind::IDENT)?.string();
                // the final name should be: struct 'name'
                let final_name = self.alloc(String::from("struct "));
                final_name.push_str(name);
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
                            name: final_name,
                            fields: fields,
                        }
                    }
                    _ => TypeExpression::NamedStruct(final_name),
                }
            }
            (TokenKind::IDENT, location) => TypeExpression::Named(location.src),
            (TokenKind::TYPEOF, _) => {
                self.expect(TokenKind::LPAREN)?;
                let expression = self.expression()?;
                self.expect(TokenKind::RPAREN)?;
                TypeExpression::Typeof(self.alloc(expression))
            }
            (x, _) => TypeExpression::Primitive(x),
        };

        while self.peek() == TokenKind::MUL {
            self.next();
            let allocation = &*self.alloc(type_expression);
            type_expression = TypeExpression::Pointer(allocation);
        }
        Ok(self.alloc(type_expression))
    }
}
