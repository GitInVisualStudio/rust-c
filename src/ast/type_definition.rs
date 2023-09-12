use crate::{
    lexer::{tokens::Token, LexerError},
    parser::{scope::IScope, Parse, Parser},
};

use super::{data_type::DataType, type_expression::TypeExpression, ASTNode};

#[derive(Debug, Clone)]
pub struct TypeDefinition {
    name: String,
    data_type: DataType,
}

impl ASTNode for TypeDefinition {}

impl Parse<TypeDefinition> for Parser<'_> {
    fn parse(&mut self) -> Result<TypeDefinition, LexerError> {
        self.expect(Token::TYPEDEF)?;

        let expression: TypeExpression = self.parse()?;
        let name = self.expect(Token::IDENT)?.to_string();

        if self.scope.contains::<TypeDefinition>(&name) {
            return self.error(format!("Type {} already defined!", name));
        }

        let result = TypeDefinition {
            data_type: expression.data_type(),
            name,
        };
        self.scope.add(result.clone());
        Ok(result)
    }
}

impl TypeDefinition {
    pub fn data_type(&self) -> DataType {
        self.data_type.clone()
    }
    pub fn name(&self) -> &String {
        &self.name
    }
}
