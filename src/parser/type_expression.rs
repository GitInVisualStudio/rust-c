use std::rc::Rc;

use crate::{
    lexer::tokens::Token,
    parser::{type_definition::TypeDefinition, variable::DataType, ASTNode},
};

use super::scope::IScope;

#[derive(Debug)]
pub struct TypeExpression {
    data_type: DataType,
}

impl ASTNode for TypeExpression {
    fn parse(
        lexer: &mut crate::lexer::Lexer,
        scope: &mut crate::parser::scope::Scope,
    ) -> Result<std::rc::Rc<Self>, crate::lexer::LexerError>
    where
        Self: Sized,
    {
        let data_type = match lexer.next() {
            Token::INT => DataType::INT,
            Token::CHAR => DataType::CHAR,
            Token::LONG => DataType::LONG,
            Token::VOID => DataType::VOID,
            Token::IDENT => {
                let name = lexer.last_string().to_string();
                let typedef: Option<&TypeDefinition> = scope.get(&name);
                if typedef.is_none() {
                    return lexer.error(format!("was not able to find type: {}", name));
                }
                typedef.unwrap().data_type()
            }
            x => panic!(
                "Was not able to parse data type of type expression! {:?}",
                x
            ),
        };
        if lexer.peek() == Token::MUL {
            lexer.next();
            let data_type = Rc::new(data_type);
            return Ok(Rc::new(TypeExpression {
                data_type: DataType::PTR(data_type),
            }));
        }
        Ok(Rc::new(TypeExpression {
            data_type: data_type,
        }))
    }

    fn generate(
        &self,
        _: &mut crate::parser::generator::Generator,
    ) -> Result<usize, std::io::Error> {
        todo!()
    }
}

impl TypeExpression {
    pub fn data_type(&self) -> DataType {
        self.data_type.clone()
    }
}
