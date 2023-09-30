use std::rc::Rc;

use crate::{
    lexer::{tokens::Token, LexerError},
    parser::{Parse, Parser},
};

use super::{
    data_type::DataType,
    expression::{Expression, UnaryOps},
    ASTNode,
};

#[derive(Debug)]
pub enum Assignment {
    VariableAssignment {
        stack_offset: usize,
        data_type: DataType,
        expression: Expression,
    },
    PtrAssignment {
        value: Expression,
        address: Rc<Expression>,
    },
    ArrayAssignment {
        index: Rc<Expression>,
        value: Expression,
        address: Rc<Expression>,
    },
    FieldAssignment {
        offset: usize,
        address: Rc<Expression>,
        value: Expression,
    },
}

impl ASTNode for Assignment {}

impl Parse<Assignment> for Parser<'_> {
    fn parse(&mut self) -> Result<Assignment, LexerError> {
        self.expect(Token::ASSIGN)?;
        let expr = self.assignee.as_ref().unwrap().clone();
        Ok(match expr.as_ref() {
            Expression::NamedVariable {
                stack_offset,
                data_type,
            } => {
                let expression: Expression = self.parse()?;
                self.check_data_types(&data_type, &expression.data_type())?;
                Assignment::VariableAssignment {
                    stack_offset: *stack_offset,
                    expression: expression,
                    data_type: data_type.clone(),
                }
            }
            Expression::Indexing { index, operand } => {
                let expression: Expression = self.parse()?;
                if let DataType::PTR(base) = operand.data_type() {
                    self.check_data_types(&base, &expression.data_type())?;
                }
                Assignment::ArrayAssignment {
                    index: index.clone(),
                    value: expression,
                    address: operand.clone(),
                }
            }
            Expression::Unary {
                expression: address,
                operation,
            } => match operation {
                UnaryOps::DEREF => {
                    let expression: Expression = self.parse()?;
                    if let DataType::PTR(base) = address.data_type() {
                        self.check_data_types(&base, &expression.data_type())?;
                    }
                    Assignment::PtrAssignment {
                        address: address.clone(),
                        value: expression,
                    }
                }
                _ => {
                    self.error("can only assing expression to a variable or pointer!".to_string())?
                }
            },
            Expression::FieldAccess {
                offset,
                data_type,
                operand,
            } => {
                let expression: Expression = self.parse()?;
                self.check_data_types(&data_type, &expression.data_type())?;
                Assignment::FieldAssignment {
                    offset: *offset,
                    address: operand.clone(),
                    value: expression,
                }
            }
            _ => self.error("Cannot assign expression to non variable!".to_string())?,
        })
    }
}

impl Assignment {
    pub fn data_type(&self) -> DataType {
        match self {
            Assignment::VariableAssignment {
                stack_offset: _,
                expression: _,
                data_type,
            } => data_type.clone(),
            Assignment::PtrAssignment { value: _, address } => match address.data_type() {
                DataType::PTR(x) => x.as_ref().clone(),
                x => x.clone(),
            },
            Assignment::ArrayAssignment {
                index: _,
                value: _,
                address,
            } => match address.data_type() {
                DataType::PTR(x) => x.as_ref().clone(),
                x => x.clone(),
            },
            Assignment::FieldAssignment {
                offset: _,
                address: _,
                value,
            } => value.data_type(),
        }
    }
}
