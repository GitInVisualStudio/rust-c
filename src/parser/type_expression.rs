use std::rc::Rc;

use crate::{
    lexer::{tokens::Token, Lexer, LexerError},
    parser::{
        data_type::{DataType, Struct},
        type_definition::TypeDefinition,
        variable::Variable,
        ASTNode,
    },
};

use super::scope::{IScope, Scope};

#[derive(Debug)]
pub struct TypeExpression {
    data_type: DataType,
}

impl ASTNode for TypeExpression {
    fn parse(lexer: &mut Lexer, scope: &mut Scope) -> Result<Rc<Self>, LexerError>
    where
        Self: Sized,
    {
        let data_type = match lexer.next() {
            Token::INT => DataType::INT,
            Token::CHAR => DataType::CHAR,
            Token::LONG => DataType::LONG,
            Token::VOID => DataType::VOID,
            Token::STRUCT => Self::parse_struct(lexer, scope)?,
            Token::IDENT => {
                let name = lexer.last_string().to_string();
                let typedef: Option<Rc<TypeDefinition>> = scope.get(&name);
                if typedef.is_none() {
                    return lexer.error(format!("was not able to find type: {}", name));
                }
                typedef.unwrap().data_type()
            }
            x => lexer.error(format!(
                "Was not able to parse data type of type expression! {:?}",
                x,
            ))?,
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
        Ok(0)
    }
}

impl TypeExpression {
    pub fn data_type(&self) -> DataType {
        self.data_type.clone()
    }

    fn parse_struct(lexer: &mut Lexer, scope: &mut Scope) -> Result<DataType, LexerError> {
        let name = lexer.expect(Token::IDENT)?.to_string();
        match lexer.peek() {
            Token::LCURL => {
                lexer.expect(Token::LCURL)?;
                let mut fields: Vec<Variable> = Vec::new();
                let mut offset = 0;
                while lexer.peek() != Token::RCURL {
                    let type_expression = TypeExpression::parse(lexer, scope)?;
                    let name = lexer.expect(Token::IDENT)?.to_string();
                    let field = Variable::new(&name, type_expression.data_type(), offset);
                    if fields.iter().find(|x| x.name() == &name).is_some() {
                        return lexer.error(format!("Field with name {} already exists!", name));
                    }
                    offset += field.data_type().size();
                    fields.push(field);
                    lexer.expect(Token::SEMIC)?;
                }
                lexer.next();

                let contains: Option<Rc<Struct>> = scope.get(&name);
                if let Some(_) = contains {
                    return lexer.error(format!("Struct '{}' already defined!", name));
                }

                let result = Rc::new(Struct::new(name, fields));
                scope.add(result.clone());
                Ok(DataType::STRUCT(result))
            }
            Token::IDENT => {
                let contains: Option<Rc<Struct>> = scope.get(&name);
                if let Some(x) = contains {
                    return Ok(DataType::STRUCT(x));
                }
                lexer.error(format!("No struct wit name '{}' found!", name))
            }
            _ => {
                let contains: Option<Rc<Struct>> = scope.get(&name);
                if let Some(x) = contains {
                    return Ok(DataType::STRUCT(x))
                }
                lexer.error(format!("Cannot find struct with name: '{}'", name))
            },
        }
    }
}
