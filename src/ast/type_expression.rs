use std::rc::Rc;

use crate::{
    lexer::{tokens::Token, LexerError},
    parser::{scope::IScope, Parse, Parser},
};

use super::{
    data_type::{DataType, Struct},
    type_definition::TypeDefinition,
    variable::Variable, ASTNode,
};

#[derive(Debug)]
pub struct TypeExpression {
    data_type: DataType,
}

impl ASTNode for TypeExpression {}

impl Parse<TypeExpression> for Parser<'_> {
    fn parse(&mut self) -> Result<TypeExpression, LexerError> {
        let mut data_type = match self.next() {
            Token::INT => DataType::INT,
            Token::CHAR => DataType::CHAR,
            Token::LONG => DataType::LONG,
            Token::VOID => DataType::VOID,
            Token::STRUCT => TypeExpression::parse_struct(self)?,
            Token::IDENT => {
                let name = self.last_string().to_string();
                let typedef: Option<&TypeDefinition> = self.scope.get(&name);
                if typedef.is_none() {
                    return self.error(format!("was not able to find type: {}", name));
                }
                typedef.unwrap().data_type()
            }
            x => self.error(format!(
                "Was not able to parse data type of type expression! {:?}",
                x,
            ))?,
        };
        while self.peek() == Token::MUL {
            self.next();
            data_type = DataType::PTR(Rc::new(data_type));
        }
        Ok(TypeExpression {
            data_type: data_type,
        })
    }
}

impl TypeExpression {
    pub fn data_type(&self) -> DataType {
        self.data_type.clone()
    }

    fn parse_struct(parser: &mut Parser) -> Result<DataType, LexerError> {
        let name = parser.expect(Token::IDENT)?.to_string();
        match parser.peek() {
            Token::LCURL => {
                parser.expect(Token::LCURL)?;
                let mut fields: Vec<Variable> = Vec::new();
                let mut offset = 0;
                while parser.peek() != Token::RCURL {
                    let type_expression: TypeExpression = parser.parse()?;
                    let name = parser.expect(Token::IDENT)?.to_string();
                    let field = Variable::new(&name, type_expression.data_type(), offset);
                    if fields.iter().find(|x| x.name() == &name).is_some() {
                        return parser.error(format!("Field with name {} already exists!", name));
                    }
                    offset += field.data_type().size();
                    fields.push(field);
                    parser.expect(Token::SEMIC)?;
                }
                parser.next();

                if parser.scope.contains::<Rc<Struct>>(&name) {
                    return parser.error(format!("Struct '{}' already defined!", name));
                }

                let result = Rc::new(Struct::new(name, fields));
                parser.scope.add(result.clone());
                Ok(DataType::STRUCT(result))
            }
            Token::IDENT => {
                let contains: Option<&Rc<Struct>> = parser.scope.get(&name);
                if let Some(x) = contains {
                    return Ok(DataType::STRUCT(x.clone()));
                }
                parser.error(format!("No struct wit name '{}' found!", name))
            }
            _ => {
                let contains: Option<&Rc<Struct>> = parser.scope.get(&name);
                if let Some(x) = contains {
                    return Ok(DataType::STRUCT(x.clone()));
                }
                parser.error(format!("Cannot find struct with name: '{}'", name))
            }
        }
    }
}
