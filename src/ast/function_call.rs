use std::rc::Rc;

use derive_getters::Getters;

use crate::{
    lexer::{tokens::Token, LexerError},
    parser::{scope::IScope, Parse, Parser},
};

use super::{data_type::DataType, expression::Expression, function::Function, ASTNode};

#[derive(Debug, Getters)]
pub struct FunctionCall {
    name: String,
    parameter: Vec<Expression>,
    data_types: Vec<DataType>,
    return_type: DataType,
}

impl ASTNode for FunctionCall {}

impl Parse<FunctionCall> for Parser<'_> {
    fn parse(&mut self) -> Result<FunctionCall, LexerError> {
        let name = self.lexer.last_string().to_owned();
        let function: Option<&Rc<Function>> = self.scope.get(&name);
        if let None = function {
            return self.error(format!("Cannot call undefined function {}!", name));
        }
        let function = function.unwrap();
        let return_type = function.return_type().clone();
        let data_types: Vec<DataType> =
            function.parameter().iter().map(|x| x.data_type()).collect();

        let mut parameter: Vec<Expression> = Vec::new();
        self.expect(Token::LPAREN)?;

        while self.peek() != Token::RPAREN {
            parameter.push(self.parse()?);
            if self.peek() == Token::COMMA {
                self.next();
            }
        }
        self.next();

        if parameter.len() != data_types.len() {
            return self.error(format!("Parameter count does not match up!"));
        }

        for (parameter, passed_parameter) in data_types.iter().zip(&parameter) {
            if *parameter != passed_parameter.data_type()
                && !parameter.can_convert(passed_parameter.data_type())
            {
                return self.error(format!("Parameter type does not match up!"));
            }
        }

        let data_types = parameter.iter().map(|x| x.data_type()).collect();

        Ok(FunctionCall {
            name: name.into(),
            parameter: parameter,
            data_types: data_types,
            return_type: return_type,
        })
    }
}
