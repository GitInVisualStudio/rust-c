use std::io::Error;
use std::rc::Rc;

use super::data_type::DataType;
use super::generator::register::Reg;
use super::generator::Generator;
use super::scope::{IScope, Scope};
use super::statement_list::StatementList;
use super::type_expression::TypeExpression;
use super::variable::Variable;
use super::ASTNode;
use crate::lexer::tokens::Token;
use crate::lexer::{Lexer, LexerError};

#[derive(Debug)]
pub struct Function {
    statements: Option<Rc<StatementList>>,
    parameter: Vec<Rc<Variable>>,
    name: String,
    return_type: DataType,
    stack_size: usize,
}

impl ASTNode for Function {
    fn parse(lexer: &mut Lexer, scope: &mut Scope) -> Result<Rc<Self>, LexerError>
    where
        Self: Sized,
    {
        scope.push();
        let type_expression = TypeExpression::parse(lexer, scope)?;
        let name = lexer.expect(Token::IDENT)?.to_string();

        lexer.expect(Token::LPAREN)?;
        let mut parameter: Vec<Rc<Variable>> = Vec::new();
        while lexer.peek() != Token::RPAREN {
            parameter.push(Function::parse_parameter(lexer, scope)?);
        }
        lexer.next();

        if lexer.peek() == Token::SEMIC {
            lexer.next();
            let result = Rc::new(Function {
                stack_size: 0,
                statements: None,
                name: name,
                parameter: parameter,
                return_type: type_expression.data_type(),
            });
            result.valid(lexer, scope)?;
            scope.pop();
            scope.add(result.clone());
            return Ok(result);
        }

        let statements = StatementList::parse(lexer, scope)?;

        let result = Rc::new(Function {
            stack_size: statements.stack_size(),
            statements: Some(statements),
            name: name,
            parameter: parameter,
            return_type: type_expression.data_type(),
        });

        result.valid(lexer, scope)?;

        scope.pop();
        scope.add(result.clone());
        Ok(result)
    }

    fn generate(&self, gen: &mut Generator) -> Result<usize, Error> {
        if self.statements.is_none() {
            return Ok(0);
        }
        gen.emit(&format!("{}:\n", self.name))?;
        gen.push_stack(self.stack_size)?;

        //push parameter onto the local stack
        for (index, parameter) in self.parameter.iter().enumerate() {
            Reg::set_size(parameter.data_type().size());
            gen.mov(
                Reg::get_parameter_index(index),
                Reg::STACK {
                    offset: parameter.offset(),
                },
            )?;
        }

        self.statements.as_ref().unwrap().generate(gen)?;
        Ok(0)
    }
}

impl Function {
    fn valid(&self, lexer: &mut Lexer, scope: &mut Scope) -> Result<bool, LexerError> {
        // check if function already exists
        let contains: Option<&Function> = scope.get(&self.name);
        if let Some(x) = contains {
            if x.statements.is_some() && self.statements.is_some() {
                return lexer.error(format!("Function {} already exists!", &self.name));
            }
            for (other, own) in self.parameter.iter().zip(&x.parameter) {
                if other.data_type() != own.data_type() || x.parameter.len() != self.parameter.len()
                {
                    return lexer.error(format!(
                        "Declaration is incompatible with other declaration!"
                    ));
                }
            }
        }
        Ok(true)
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    fn parse_parameter(lexer: &mut Lexer, scope: &mut Scope) -> Result<Rc<Variable>, LexerError> {
        let type_expression = TypeExpression::parse(lexer, scope)?;
        let name = lexer.expect(Token::IDENT)?.to_string();
        let var = Variable::new(&name, type_expression.data_type(), scope.stack_size());
        let var = Rc::new(var);

        let contains: Option<&Variable> = scope.get(&name);
        if let Some(_) = contains {
            return lexer.error(format!(
                "Parameter with name {} already declared in scope!",
                name
            ));
        }
        scope.add(var.clone());

        if lexer.peek() == Token::COMMA {
            lexer.next();
        }

        Ok(var)
    }

    pub fn parameter(&self) -> &Vec<Rc<Variable>> {
        &self.parameter
    }

    pub fn return_type(&self) -> DataType {
        self.return_type.clone()
    }
}
