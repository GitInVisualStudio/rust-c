use crate::{error::Error, lexer::tokens::TokenKind, parser::Parser};

use super::{expression::Expression, ASTNode};

#[derive(Debug)]
pub enum Assignment<'a> {
    VariableAssignment {
        name: &'a str,
        expression: Expression<'a>,
    },
    PtrAssignment {
        value: Expression<'a>,
        address: &'a Expression<'a>,
    },
    ArrayAssignment {
        index: &'a Expression<'a>,
        value: Expression<'a>,
        address: &'a Expression<'a>,
    },
    FieldAssignment {
        name: &'a str,
        value: Expression<'a>,
        address: &'a Expression<'a>,
    },
}

impl ASTNode for Assignment<'_> {}

impl<'a> Parser<'a> {
    pub fn assignment(&mut self) -> Result<Assignment<'a>, Error<'a>> {
        self.expect(TokenKind::ASSIGN)?;
        let expr = self.assignee.unwrap();
        Ok(match expr {
            Expression::NamedVariable { name } => {
                let expression = self.expression()?;
                Assignment::VariableAssignment {
                    name: name,
                    expression: expression,
                }
            }
            Expression::Indexing { index, operand } => {
                let expression = self.expression()?;
                Assignment::ArrayAssignment {
                    index: index,
                    value: expression,
                    address: operand,
                }
            }
            Expression::Unary {
                expression: address,
                ..
            } => {
                let expression = self.expression()?;
                Assignment::PtrAssignment {
                    address: address,
                    value: expression,
                }
            }
            Expression::FieldAccess { name, operand } => {
                let expression = self.expression()?;
                Assignment::FieldAssignment {
                    name: name,
                    address: operand,
                    value: expression,
                }
            }
            _ => {
                return Err(Error::UnableToAssign {
                    location: self.current().1,
                })
            }
        })
    }
}
