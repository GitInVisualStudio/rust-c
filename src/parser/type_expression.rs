use std::rc::Rc;

use crate::{parser::{variable::DataType, ASTNode}, lexer::tokens::Token};

#[derive(Debug)]
pub struct TypeExpression {
    data_type: DataType,
}

impl ASTNode for TypeExpression {
    fn parse(
        lexer: &mut crate::lexer::Lexer,
        _: &mut crate::parser::scope::Scope,
    ) -> Result<std::rc::Rc<Self>, crate::lexer::LexerError>
    where
        Self: Sized,
    {
        let data_type = match lexer.next() {
            Token::INT => DataType::INT,
            Token::CHAR => DataType::CHAR,
            Token::LONG => DataType::LONG,
            Token::VOID => DataType::VOID,
            _ => panic!("Was not able to parse data type of type expression!")
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