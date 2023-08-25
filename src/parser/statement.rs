use std::io::Error;
use std::rc::Rc;

use crate::lexer::{Lexer, LexerError};
use crate::lexer::tokens::Token;
use super::ASTNode;
use super::expression::Expression;
use super::generator::Generator;
use super::if_statement::IfStatement;
use super::scope::{Scope, IScope};
use super::variable::Variable;


#[derive(Debug)]
pub enum Statement {
    Return {
        expression: Option<Rc<dyn ASTNode>>
    },
    VariableDeclaration {
        variable: Rc<Variable>,
        expression: Option<Rc<dyn ASTNode>>
    },
    IfStatement(Rc<IfStatement>)
}

impl ASTNode for Statement {
    fn parse(lexer: &mut Lexer, scope: &mut Scope) -> Result<Rc<Self>, LexerError> where Self: Sized {
        let result = match lexer.peek() {
            Token::INT => Self::parse_variable_declaration(lexer, scope),
            Token::RETURN => Self::parse_return(lexer, scope),
            Token::IF => {
                let value = IfStatement::parse(lexer, scope)?;
                let value = Rc::new(Statement::IfStatement(value));
                return Ok(value)
            }
            x => lexer.error(format!("Cannot parse statement: {:?}", x))
        }?;
        lexer.expect(Token::SEMIC)?;
        Ok(result)
    }

    fn generate(&self, gen: &mut Generator) -> Result<usize, Error> {
        match self {
            Statement::Return { expression } => {
                if expression.is_some() {
                    expression.as_ref().unwrap().generate(gen)?;
                }
                gen.pop_stack()?;
                gen.emit("\tret\n".to_string())?;
                Ok(0)
            },
            Statement::VariableDeclaration { variable, expression } => {
                if expression.is_some() {
                    expression.as_ref().unwrap().generate(gen)?;
                    gen.emit_ins("mov ", "%eax", format!("-{}(%rbp)", variable.offset()).as_str())?;
                }
                Ok(0)
            },
            Statement::IfStatement(if_statement) => if_statement.generate(gen),
        }
    }

}

impl Statement {
    fn parse_variable_declaration(lexer: &mut Lexer, scope: &mut Scope) -> Result<Rc<Self>, LexerError> {
        lexer.expect(Token::INT)?;
        
        let name = lexer.expect(Token::IDENT)?.to_string();
        let var = Variable::new(&name, super::variable::DataType::INT, scope.stack_size());
        let var = Rc::new(var);

        let contains: Option<&Variable> = scope.get(&name);
        if let Some(_) = contains {
            return lexer.error(format!("Variable {} already declared in scope!", name))
        }
        scope.add(var.clone());
        
        Ok(Rc::new(match lexer.peek() {
            Token::ASSIGN => {
                lexer.next();
                let expression = Expression::parse(lexer, scope)?;
                Statement::VariableDeclaration { variable: var, expression: Some(expression) }
            }
            _ => Statement::VariableDeclaration { variable: var, expression: None }
        }))
    }

    fn parse_return(lexer: &mut Lexer, scope: &mut Scope) -> Result<Rc<Self>, LexerError> {
        lexer.expect(Token::RETURN)?;
        let expression = Expression::parse(lexer, scope)?;
        Ok(Rc::new(Statement::Return {
            expression: Some(expression)
        }))
    }
}