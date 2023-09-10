use std::io::Error;
use std::rc::Rc;

use super::assignment::Assignment;
use super::data_type::{DataType, Struct};
use super::expression::Expression;
use super::for_statement::ForStatement;
use super::generator::register::Reg;
use super::generator::Generator;
use super::if_statement::IfStatement;
use super::scope::{IScope, Scope};
use super::statement_list::StatementList;
use super::type_definition::TypeDefinition;
use super::type_expression::TypeExpression;
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
    SingleExpression {
        expression: Rc<Expression>,
    },
    VariableDeclaration {
        variable: Rc<Variable>,
        expression: Option<Assignment>,
    },
    StatementList(Rc<StatementList>),
    IfStatement(Rc<IfStatement>),
    ForStatement(Rc<ForStatement>),
    WhileStatement(Rc<WhileStatement>),
    TypeDefinition(Rc<TypeDefinition>),
    Continue {
        label_index: usize,
    },
    Break {
        label_index: usize,
    },
    Empty,
}

impl ASTNode for Statement {
    fn parse(lexer: &mut Lexer, scope: &mut Scope) -> Result<Rc<Self>, LexerError>
    where
        Self: Sized,
    {
        let result = match lexer.peek() {
            Token::CONTINUE => {
                if Generator::label_index() == 0 {
                    lexer.error("Continue may only be used inside a loop!".to_string())?;
                }
                lexer.next();
                Ok(Rc::new(Self::Continue {
                    label_index: Generator::label_index(),
                }))
            }
            Token::BREAK => {
                if Generator::label_index() == 0 {
                    lexer.error("Break may only be used inside a loop!".to_string())?;
                }
                lexer.next();
                Ok(Rc::new(Self::Break {
                    label_index: Generator::label_index(),
                }))
            }
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
            Token::INT | Token::CHAR | Token::LONG | Token::VOID | Token::STRUCT => {
                Self::parse_variable_declaration(lexer, scope)
            }
            Token::IDENT => {
                let name = lexer.peek_str();
                if scope.contains::<Struct>(name) {
                    Self::parse_variable_declaration(lexer, scope)
                } else if scope.contains::<TypeDefinition>(name) {
                    Self::parse_variable_declaration(lexer, scope)
                } else {
                    Ok(Rc::new(Self::SingleExpression {
                        expression: Expression::parse(lexer, scope)?,
                    }))
                }
            }
            Token::TYPEDEF => Ok(Rc::new(Self::TypeDefinition(TypeDefinition::parse(
                lexer, scope,
            )?))),
            Token::SEMIC => Ok(Rc::new(Self::Empty)),
            Token::LCURL => {
                return Ok(Rc::new(Self::StatementList(StatementList::parse(
                    lexer, scope,
                )?)))
            }
            _ => Ok(Rc::new(Self::SingleExpression {
                expression: Expression::parse(lexer, scope)?,
            })),
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
                gen.mov(Reg::current(), Reg::RAX)?;
                gen.ret()
            }
            Statement::VariableDeclaration {
                variable: _,
                expression,
            } => {
                if expression.is_some() {
                    expression.as_ref().unwrap().generate(gen)?;
                }
                Ok(0)
            }
            Statement::IfStatement(statement) => statement.generate(gen),
            Statement::SingleExpression { expression } => expression.generate(gen),
            Statement::ForStatement(statement) => statement.generate(gen),
            Statement::WhileStatement(while_statement) => while_statement.generate(gen),
            Statement::Empty => Ok(0),
            Statement::Continue { label_index } => {
                let (_, _, condition) = Generator::generate_label_names(*label_index);
                gen.jmp(&condition)
            }
            Statement::Break { label_index } => {
                let (_, end, _) = Generator::generate_label_names(*label_index);
                gen.jmp(&end)
            }
            Statement::StatementList(list) => list.generate(gen),
            Statement::TypeDefinition(typedef) => typedef.generate(gen),
        }
    }
}

impl Statement {
    fn parse_variable_declaration(
        lexer: &mut Lexer,
        scope: &mut Scope,
    ) -> Result<Rc<Self>, LexerError> {
        let expression = TypeExpression::parse(lexer, scope)?;

        if lexer.peek() == Token::SEMIC {
            return Ok(Rc::new(Self::SingleExpression {
                expression: Rc::new(Expression::TypeExpression(expression)),
            }));
        }

        let name = lexer.expect(Token::IDENT)?.to_string();
        let var = Variable::new(&name, expression.data_type(), scope.stack_size());
        let mut var = Rc::new(var);

        if scope.contains::<Variable>(&name) {
            return lexer.error(format!("Variable {} already declared in scope!", name));
        }

        if lexer.peek() == Token::LBRACE {
            lexer.next();
            lexer.expect(Token::RBRACE)?;
            var = Rc::new(Variable::new(
                &name,
                DataType::PTR(Rc::new(expression.data_type())),
                scope.stack_size(),
            ));
            scope.add(var.clone());
        } else {
            scope.add(var.clone());
        }

        Ok(Rc::new(match lexer.peek() {
            Token::ASSIGN => {
                lexer.next();
                let expression = Expression::parse(lexer, scope)?;
                if !expression.data_type().can_convert(var.data_type()) {
                    lexer.error(format!(
                        "cannot convert from {:?} to {:?}!",
                        expression.data_type(),
                        var.data_type()
                    ))?
                }
                Statement::VariableDeclaration {
                    expression: Some(Assignment::VariableAssignment {
                        stack_offset: var.offset(),
                        expression: expression,
                    }),
                    variable: var,
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
        if lexer.peek() == Token::SEMIC {
            return Ok(Rc::new(Statement::Return { expression: None }));
        }
        let expression = Expression::parse(lexer, scope)?;
        Ok(Rc::new(Statement::Return {
            expression: Some(expression),
        }))
    }
}
