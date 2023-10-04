use crate::{error::Error, lexer::tokens::TokenKind, parser::Parser, visitor::Visitable};

use super::type_expression::TypeExpression;

#[derive(Debug)]
pub struct TypeDefinition<'a> {
    pub(crate) name: &'a str,
    pub(crate) expression: &'a TypeExpression<'a>,
}

impl Visitable for TypeDefinition<'_> {}

impl<'a> Parser<'a> {
    pub fn type_def(&mut self) -> Result<&'a TypeDefinition<'a>, Error<'a>> {
        self.expect(TokenKind::TYPEDEF)?;

        let expression = self.type_expression()?;
        let name = self.expect(TokenKind::IDENT)?.string();

        Ok(self.alloc(TypeDefinition { expression, name }))
    }
}
