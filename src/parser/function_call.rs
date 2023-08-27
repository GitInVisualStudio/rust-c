use std::rc::Rc;

use crate::lexer::{tokens::Token, Lexer};

use super::{
    expression::Expression,
    function::Function,
    scope::{IScope, Scope},
    variable::DataType,
    ASTNode, generator::Generator,
};

#[derive(Debug)]
pub struct FunctionCall {
    name: String,
    parameter: Vec<Rc<Expression>>,
}

impl ASTNode for FunctionCall {
    fn parse(
        lexer: &mut crate::lexer::Lexer,
        scope: &mut super::scope::Scope,
    ) -> Result<Rc<Self>, crate::lexer::LexerError>
    where
        Self: Sized,
    {
        todo!();
    }

    fn generate(&self, gen: &mut super::generator::Generator) -> Result<usize, std::io::Error> {
        // store parameter in registers
        for (index, parameter) in self.parameter.iter().enumerate() {
            parameter.generate(gen)?;
            gen.emit_ins("mov ", "%eax", Generator::get_register(index))?;
        }
        gen.emit(format!("\tcall \t{}\n", self.name))?;
        Ok(0)
    }
}

impl FunctionCall {
    pub fn parse_name(
        name: String,
        lexer: &mut Lexer,
        scope: &mut Scope,
    ) -> Result<Rc<Self>, crate::lexer::LexerError> {
        let function: Option<&Function> = scope.get(&name);
        if let None = function {
            return lexer.error(format!("Cannot call undefined function {}!", name));
        }

        let function: Vec<DataType> = function
            .unwrap()
            .parameter()
            .iter()
            .map(|x| x.data_type())
            .collect();

        let mut parameter: Vec<Rc<Expression>> = Vec::new();
        lexer.expect(Token::LPAREN)?;

        while lexer.peek() != Token::RPAREN {
            parameter.push(Expression::parse(lexer, scope)?);
            if lexer.peek() == Token::COMMA {
                lexer.next();
            }
        }
        lexer.next();

        if parameter.len() != function.len() {
            return lexer.error(format!("Parameter count does not match up!"));
        }

        for (parameter, passed_parameter) in function.iter().zip(&parameter) {
            if *parameter != passed_parameter.data_type() {
                return lexer.error(format!("Parameter type does not match up!"));
            }
        }

        Ok(Rc::new(FunctionCall {
            name: name,
            parameter: parameter,
        }))
    }
}
