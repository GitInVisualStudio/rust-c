use std::{io::Error, rc::Rc};

use crate::{
    lexer::{tokens::Token, Lexer, LexerError},
    parser::generator::register::Reg,
};

use super::{
    data_type::{DataType, Struct},
    expression::{Expression, UnaryOps},
    generator::Generator,
    ASTNode,
};

#[derive(Debug)]
pub enum Assignment {
    VariableAssignment {
        stack_offset: usize,
        expression: Rc<Expression>,
    },
    PtrAssignment {
        value: Rc<Expression>,
        address: Rc<Expression>,
    },
    ArrayAssignment {
        index: Rc<Expression>,
        value: Rc<Expression>,
        address: Rc<Expression>,
    },
    FieldAssignment {
        offset: usize,
        address: Rc<Expression>,
        value: Rc<Expression>,
    },
}

impl ASTNode for Assignment {
    fn parse(
        _: &mut crate::lexer::Lexer,
        _: &mut super::scope::Scope,
    ) -> Result<Rc<Self>, crate::lexer::LexerError>
    where
        Self: Sized,
    {
        todo!()
    }

    fn generate(&self, gen: &mut super::generator::Generator) -> Result<usize, std::io::Error> {
        match self {
            Assignment::VariableAssignment {
                stack_offset,
                expression,
            } => match expression.data_type() {
                DataType::STRUCT(_) => {
                    expression.generate(gen)?;
                    let from = Reg::push();
                    Struct::mov(gen, from, *stack_offset, expression.data_type())?;
                    Reg::pop();
                    Ok(0)
                }
                _ => {
                    Reg::set_size(self.data_type().size());
                    expression.generate(gen)?;
                    gen.mov(
                        Reg::current(),
                        Reg::STACK {
                            offset: *stack_offset,
                        },
                    )
                }
            },
            Assignment::PtrAssignment { value, address } => {
                Reg::set_size(self.data_type().size());
                address.generate(gen)?;
                let address = Reg::push();
                value.generate(gen)?;
                let value = Reg::current();

                Reg::set_size(8);
                let address = format!("({})", address);

                Reg::set_size(self.data_type().size());
                let result = gen.emit(&format!("\tmov \t{}, {}\n", value, address));
                Reg::pop();
                result
            }
            Assignment::ArrayAssignment {
                index,
                value,
                address,
            } => {
                value.generate(gen)?;
                let value = Reg::push();

                address.generate(gen)?;
                let address = Reg::push();

                index.generate(gen)?;
                let index = Reg::current();

                gen.mul(Reg::IMMEDIATE(self.data_type().size() as i64), index)?;
                Reg::set_size(8);
                gen.add(index, address)?;
                let address = format!("({})", address);
                Reg::set_size(self.data_type().size());
                let result = gen.emit(&format!("\tmov \t{}, {}\n", value, address));

                Reg::pop();
                Reg::pop();
                result
            }
            Assignment::FieldAssignment {
                offset,
                address,
                value,
            } => {
                address.generate(gen)?;
                let address = Reg::push();
                value.generate(gen)?;
                let value = Reg::current();
                Reg::set_size(8);
                gen.add(Reg::IMMEDIATE(*offset as i64), address)?;
                let address = format!("({})", address);
                Reg::set_size(self.data_type().size());
                let result = gen.emit(&format!("\tmov \t{}, {}\n", value, address));
                Reg::pop();
                result
            }
        }
    }
}

impl Assignment {
    pub fn data_type(&self) -> DataType {
        match self {
            Assignment::VariableAssignment {
                stack_offset: _,
                expression,
            } => expression.data_type(),
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

    fn check_data_types(
        from: &DataType,
        to: &DataType,
        lexer: &mut Lexer,
    ) -> Result<bool, LexerError> {
        if *from != *to && !from.can_convert(to.clone()) {
            return lexer.error(format!("Cannot assign {:?} to {:?}!", from, to));
        }
        Ok(true)
    }

    pub fn parse(
        base: &Rc<Expression>,
        lexer: &mut crate::lexer::Lexer,
        scope: &mut super::scope::Scope,
    ) -> Result<Rc<Self>, crate::lexer::LexerError> {
        lexer.expect(Token::ASSIGN)?;
        Ok(Rc::new(match base.as_ref() {
            Expression::NamedVariable {
                stack_offset,
                data_type,
            } => {
                let expression = Expression::parse(lexer, scope)?;
                Self::check_data_types(data_type, &expression.data_type(), lexer)?;
                Self::VariableAssignment {
                    stack_offset: *stack_offset,
                    expression: expression,
                }
            }
            Expression::Indexing { index, operand } => {
                let expression = Expression::parse(lexer, scope)?;
                if let DataType::PTR(base) = operand.data_type() {
                    Self::check_data_types(&base, &expression.data_type(), lexer)?;
                }
                Self::ArrayAssignment {
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
                    let expression = Expression::parse(lexer, scope)?;
                    if let DataType::PTR(base) = address.data_type() {
                        Self::check_data_types(&base, &expression.data_type(), lexer)?;
                    }
                    Self::PtrAssignment {
                        address: address.clone(),
                        value: expression,
                    }
                }
                _ => lexer
                    .error("can only assing expression to a variable or pointer!".to_string())?,
            },
            Expression::FieldAccress {
                offset,
                data_type,
                operand,
            } => {
                let expression = Expression::parse(lexer, scope)?;
                Self::check_data_types(data_type, &expression.data_type(), lexer)?;
                Self::FieldAssignment {
                    offset: *offset,
                    address: operand.clone(),
                    value: expression,
                }
            }
            _ => lexer.error("Cannot assign expression to non variable!".to_string())?,
        }))
    }
}
