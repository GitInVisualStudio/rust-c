use std::rc::Rc;

use crate::lexer::{tokens::Token, Lexer};

use super::{
    expression::Expression,
    function::Function,
    scope::{IScope, Scope},
    variable::DataType,
    ASTNode, generator::{Generator, register::Reg},
};

#[derive(Debug)]
pub struct FunctionCall {
    name: String,
    parameter: Vec<Rc<Expression>>,
}

impl ASTNode for FunctionCall {
    fn parse(
        _: &mut crate::lexer::Lexer,
        _: &mut super::scope::Scope,
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
            gen.mov(Reg::current(), Reg::get_parameter_index(index))?;
        }
        
        Reg::set_size(8);
        let prev = Reg::current();
        while Reg::current() != Reg::R10 {
            gen.emit_sins("push", Reg::pop())?;
        }
        gen.emit_sins("push", Reg::current())?;

        gen.call(&self.name)?;

        Reg::set_size(8);
        while Reg::current() != prev {
            gen.emit_sins("pop ", Reg::push())?;
        }
        gen.emit_sins("pop ", Reg::current())?;
        gen.mov(Reg::RAX, Reg::current())
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
