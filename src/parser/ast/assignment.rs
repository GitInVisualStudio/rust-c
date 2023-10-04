use crate::{error::Error, lexer::tokens::TokenKind, parser::Parser, visitor::Visitable};

use super::{expression::Expression, UnaryOps};

#[derive(Debug)]
pub enum Assignment<'a> {
    VariableAssignment {
        name: &'a str,
        expression: &'a Expression<'a>,
    },
    PtrAssignment {
        value: &'a Expression<'a>,
        address: &'a Expression<'a>,
    },
    ArrayAssignment {
        index: &'a Expression<'a>,
        value: &'a Expression<'a>,
        address: &'a Expression<'a>,
    },
    FieldAssignment {
        name: &'a str,
        value: &'a Expression<'a>,
        address: &'a Expression<'a>,
    },
}

impl Visitable for Assignment<'_> {}

impl<'a> Parser<'a> {
    pub fn assignment(&mut self) -> Result<&'a Assignment<'a>, Error<'a>> {
        self.expect(TokenKind::ASSIGN)?;
        let expr = self.assignee.unwrap();
        Ok(self.bump.alloc(match expr {
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
                operation,
            } => match operation {
                UnaryOps::DEREF => {
                    let expression = self.expression()?;
                    Assignment::PtrAssignment {
                        address: address,
                        value: expression,
                    }
                }
                _ => {
                    return Err(Error::UnableToAssign {
                        location: self.current().1,
                    })
                }
            },
            Expression::FieldAccess { name, operand } => {
                let expression = self.expression()?;
                Assignment::FieldAssignment {
                    name: name,
                    address: operand,
                    value: expression,
                }
            }
            Expression::ArrowAccess { name, operand } => {
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
        }))
    }
}
