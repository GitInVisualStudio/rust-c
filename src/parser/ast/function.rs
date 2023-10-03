use crate::{error::Error, lexer::tokens::TokenKind, parser::Parser};

use super::{compound_statement::Compound, type_expression::TypeExpression, ASTNode};

#[derive(Debug)]
pub struct Function<'a> {
    pub(crate) name: &'a str,
    pub(crate) statements: Option<Compound<'a>>,
    pub(crate) parameter: Vec<(TypeExpression<'a>, &'a str)>,
    pub(crate) return_type: TypeExpression<'a>,
}

impl ASTNode for Function<'_> {}

impl<'a> Parser<'a> {
    pub fn function(&mut self) -> Result<Function<'a>, Error<'a>> {
        let return_type = self.type_expression()?;
        let name = self.expect(TokenKind::IDENT)?.string();

        self.expect(TokenKind::LPAREN)?;
        let mut parameter = Vec::new();

        while self.peek() != TokenKind::RPAREN {
            let type_expression = self.type_expression()?;
            let name = self.expect(TokenKind::IDENT)?.string();
            parameter.push((type_expression, name));
            if self.peek() == TokenKind::RPAREN {
                break;
            }
            self.expect(TokenKind::COMMA)?;
        }

        self.next();

        if self.peek() == TokenKind::SEMIC {
            self.next();
            return Ok(Function {
                statements: None,
                name,
                parameter,
                return_type,
            });
        }

        let statements = self.compound_statement()?;

        Ok(Function {
            statements: Some(statements),
            name,
            parameter,
            return_type,
        })
    }
}
