use crate::{error::Error, lexer::tokens::TokenKind, parser::Parser, visitor::Visitable};

use super::type_expression::TypeExpression;

#[derive(Debug)]
pub struct TypeDefinition<'a> {
    pub(crate) name: &'a str,
    pub(crate) expression: TypeExpression<'a>,
}

impl Visitable for TypeDefinition<'_> {}

impl<'a> Parser<'a> {
    pub fn type_def(&mut self) -> Result<TypeDefinition<'a>, Error<'a>> {
        self.expect(TokenKind::TYPEDEF)?;

        let expression = self.type_expression()?;
        let name = self.expect(TokenKind::IDENT)?.string();

        let result = TypeDefinition { expression, name };
        Ok(result)
    }
}
