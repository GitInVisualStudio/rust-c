use std::rc::Rc;

use super::assignment::Assignment;
use super::data_type::{DataType, Struct};
use super::expression::Expression;
use super::for_statement::ForStatement;
use super::if_statement::IfStatement;
use super::statement_list::StatementList;
use super::type_definition::TypeDefinition;
use super::type_expression::TypeExpression;
use super::variable::Variable;
use super::while_statement::WhileStatement;
use super::ASTNode;
use crate::lexer::tokens::Token;
use crate::lexer::LexerError;
use crate::parser::scope::IScope;
use crate::parser::{Parse, Parser};

#[derive(Debug)]
pub enum Statement {
    Return(Option<Expression>),
    SingleExpression(Expression),
    StatementList(StatementList),
    IfStatement(IfStatement),
    ForStatement(ForStatement),
    WhileStatement(WhileStatement),
    TypeDefinition(TypeDefinition),
    VariableDeclaration {
        variable: Variable,
        expression: Option<Assignment>,
    },
    Continue {
        label_index: usize,
    },
    Break {
        label_index: usize,
    },
    Empty,
}

impl ASTNode for Statement {}

impl Parse<Statement> for Parser<'_> {
    fn parse(&mut self) -> Result<Statement, LexerError> {
        let result = match self.peek() {
            Token::CONTINUE => {
                if self.label_index() == 0 {
                    self.error("Continue may only be used inside a loop!".to_string())?;
                }
                self.next();
                Ok(Statement::Continue {
                    label_index: self.label_index(),
                })
            }
            Token::BREAK => {
                if self.label_index() == 0 {
                    self.error("Break may only be used inside a loop!".to_string())?;
                }
                self.next();
                Ok(Statement::Break {
                    label_index: self.label_index(),
                })
            }
            Token::RETURN => {
                self.expect(Token::RETURN)?;
                if self.peek() == Token::SEMIC {
                    return Ok(Statement::Return(None));
                }
                let expression = self.parse()?;
                Ok(Statement::Return(Some(expression)))
            }
            Token::IF => {
                let statement = self.parse()?;
                return Ok(Statement::IfStatement(statement));
            }
            Token::FOR => {
                let statement = self.parse()?;
                return Ok(Statement::ForStatement(statement));
            }
            Token::WHILE => {
                let statement = self.parse()?;
                return Ok(Statement::WhileStatement(statement));
            }
            Token::INT | Token::CHAR | Token::LONG | Token::VOID | Token::STRUCT | Token::TYPEOF => {
                Statement::parse_variable_declaration(self)
            }
            Token::IDENT => {
                let name = self.peek_str().to_owned();
                if self.scope.contains::<Rc<Struct>>(&name) {
                    Statement::parse_variable_declaration(self)
                } else if self.scope.contains::<TypeDefinition>(&name) {
                    Statement::parse_variable_declaration(self)
                } else {
                    Ok(Statement::SingleExpression(self.parse()?))
                }
            }
            Token::TYPEDEF => {
                let def = self.parse()?;
                Ok(Statement::TypeDefinition(def))
            }
            Token::SEMIC => Ok(Statement::Empty),
            Token::LCURL => {
                let list = self.parse()?;
                return Ok(Statement::StatementList(list));
            }
            _ => {
                let expr = self.parse()?;
                Ok(Statement::SingleExpression(expr))
            }
        }?;
        self.expect(Token::SEMIC)?;
        Ok(result)
    }
}

impl Statement {
    fn parse_variable_declaration(parser: &mut Parser) -> Result<Self, LexerError> {
        let expression: TypeExpression = parser.parse()?;

        if parser.peek() == Token::SEMIC {
            return Ok(Self::SingleExpression(Expression::TypeExpression(
                expression,
            )));
        }

        let name = parser.expect(Token::IDENT)?.to_string();
        let mut var = Variable::new(&name, expression.data_type(), parser.scope.stack_size());

        if parser.scope.contains::<Variable>(&name) {
            return parser.error(format!("Variable {} already declared in scope!", name));
        }

        if parser.peek() == Token::LBRACE {
            parser.next();
            parser.expect(Token::RBRACE)?;
            var = Variable::new(
                &name,
                DataType::PTR(Rc::new(expression.data_type())),
                parser.scope.stack_size(),
            );
            parser.scope.add(var.clone());
        } else {
            parser.scope.add(var.clone());
        }

        Ok(match parser.peek() {
            Token::ASSIGN => {
                parser.next();
                let expression: Expression = parser.parse()?;
                if !expression.data_type().can_convert(var.data_type()) {
                    parser.error(format!(
                        "cannot convert from {:?} to {:?}!",
                        expression.data_type(),
                        var.data_type()
                    ))?
                }
                Statement::VariableDeclaration {
                    expression: Some(Assignment::VariableAssignment {
                        stack_offset: var.offset(),
                        expression: expression,
                        data_type: var.data_type(),
                    }),
                    variable: var,
                }
            }
            _ => Statement::VariableDeclaration {
                variable: var,
                expression: None,
            },
        })
    }
}
