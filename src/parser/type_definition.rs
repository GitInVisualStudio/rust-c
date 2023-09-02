use std::rc::Rc;

use crate::lexer::tokens::Token;

use super::{scope::IScope, type_expression::TypeExpression, data_type::DataType, ASTNode};

#[derive(Debug)]
pub struct TypeDefinition {
    name: String,
    data_type: DataType,
}

impl ASTNode for TypeDefinition {
    fn parse(
        lexer: &mut crate::lexer::Lexer,
        scope: &mut super::scope::Scope,
    ) -> Result<Rc<Self>, crate::lexer::LexerError>
    where
        Self: Sized,
    {
        lexer.expect(Token::TYPEDEF)?;

        let expression = TypeExpression::parse(lexer, scope)?;
        let name = lexer.expect(Token::IDENT)?.to_string();

        let contains: Option<Rc<TypeDefinition>> = scope.get(&name);
        if contains.is_some() {
            return lexer.error(format!("Type {} already defined!", name));
        }

        let result = Rc::new(TypeDefinition {
            data_type: expression.data_type(),
            name,
        });
        scope.add(result.clone());
        Ok(result)
    }

    fn generate(&self, _: &mut super::generator::Generator) -> Result<usize, std::io::Error> {
        Ok(0)
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
