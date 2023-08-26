use std::io::Error;
use std::rc::Rc;

use super::expression::Expression;
use super::for_statement::ForStatement;
use super::generator::Generator;
use super::if_statement::IfStatement;
use super::scope::{IScope, Scope};
use super::variable::Variable;
use super::while_statement::WhileStatement;
use super::ASTNode;
use crate::lexer::tokens::Token;
use crate::lexer::{Lexer, LexerError};

#[derive(Debug)]
pub enum Statement {
    Return {
        expression: Option<Rc<dyn ASTNode>>,
    },
    Expression {
        expression: Rc<dyn ASTNode>,
    },
    VariableDeclaration {
        variable: Rc<Variable>,
        expression: Option<Rc<dyn ASTNode>>,
    },
    IfStatement(Rc<IfStatement>),
    ForStatement(Rc<ForStatement>),
    WhileStatement(Rc<WhileStatement>),
    Empty,
}

impl ASTNode for Statement {
    fn parse(lexer: &mut Lexer, scope: &mut Scope) -> Result<Rc<Self>, LexerError>
    where
        Self: Sized,
    {
        let result = match lexer.peek() {
            Token::INT => Self::parse_variable_declaration(lexer, scope),
            Token::RETURN => Self::parse_return(lexer, scope),
            Token::IF => {
                let statement = IfStatement::parse(lexer, scope)?;
                let statement = Rc::new(Statement::IfStatement(statement));
                return Ok(statement);
            }
            Token::FOR => {
                let statement = ForStatement::parse(lexer, scope)?;
                let statement = Rc::new(Statement::ForStatement(statement));
                return Ok(statement);
            }
            Token::WHILE => {
                let statement = WhileStatement::parse(lexer, scope)?;
                let statement = Rc::new(Statement::WhileStatement(statement));
                return Ok(statement);
            }
            Token::IDENT | Token::INTLITERAL => Ok(Rc::new(Self::Expression {
                expression: Expression::parse(lexer, scope)?,
            })),
            Token::SEMIC => Ok(Rc::new(Self::Empty)),
            x => lexer.error(format!("Cannot parse statement: {:?}", x)),
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
            }
            Statement::VariableDeclaration {
                variable,
                expression,
            } => {
                if expression.is_some() {
                    expression.as_ref().unwrap().generate(gen)?;
                    gen.emit_ins(
                        "mov ",
                        "%eax",
                        format!("-{}(%rbp)", variable.offset()).as_str(),
                    )?;
                }
                Ok(0)
            }
            Statement::IfStatement(statement) => statement.generate(gen),
            Statement::Expression { expression } => expression.generate(gen),
            Statement::ForStatement(statement) => statement.generate(gen),
            Statement::WhileStatement(while_statement) => while_statement.generate(gen),
            Statement::Empty => Ok(0),
        }
    }
}

impl Statement {
    fn parse_variable_declaration(
        lexer: &mut Lexer,
        scope: &mut Scope,
    ) -> Result<Rc<Self>, LexerError> {
        lexer.expect(Token::INT)?;

        let name = lexer.expect(Token::IDENT)?.to_string();
        let var = Variable::new(&name, super::variable::DataType::INT, scope.stack_size());
        let var = Rc::new(var);

        let contains: Option<&Variable> = scope.get(&name);
        if let Some(_) = contains {
            return lexer.error(format!("Variable {} already declared in scope!", name));
        }
        scope.add(var.clone());

        Ok(Rc::new(match lexer.peek() {
            Token::ASSIGN => {
                lexer.next();
                let expression = Expression::parse(lexer, scope)?;
                Statement::VariableDeclaration {
                    variable: var,
                    expression: Some(expression),
                }
            }
            _ => Statement::VariableDeclaration {
                variable: var,
                expression: None,
            },
        }))
    }

    fn parse_return(lexer: &mut Lexer, scope: &mut Scope) -> Result<Rc<Self>, LexerError> {
        lexer.expect(Token::RETURN)?;
        let expression = Expression::parse(lexer, scope)?;
        Ok(Rc::new(Statement::Return {
            expression: Some(expression),
        }))
    }
}
