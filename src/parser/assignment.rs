use std::rc::Rc;

use crate::{lexer::tokens::Token, parser::generator::register::Reg};

use super::{
    expression::{Expression, UnaryOps},
    variable::DataType,
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
}

impl ASTNode for Assignment {
    fn parse(
        lexer: &mut crate::lexer::Lexer,
        scope: &mut super::scope::Scope,
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
            } => {
                Reg::set_size(self.data_type().size());
                expression.generate(gen)?;
                gen.mov(
                    Reg::current(),
                    Reg::STACK {
                        offset: *stack_offset,
                    },
                )
            }
            Assignment::PtrAssignment { value, address } => {
                Reg::set_size(self.data_type().size());
                let address_reg = Reg::current();
                address.generate(gen)?;
                Reg::push();
                value.generate(gen)?;
                Reg::set_size(8);
                let result = gen.emit(&format!("\tmov \t{}, ({})\n", Reg::current(), address_reg));
                Reg::pop();
                result
            }
            Assignment::ArrayAssignment {
                index,
                value,
                address,
            } => {
                index.generate(gen)?;
                let index = Reg::push();
                address.generate(gen)?;
                let address = Reg::push();
                value.generate(gen)?;
                gen.mul(Reg::IMMEDIATE(self.data_type().size() as i64), index)?;
                Reg::set_size(8);
                gen.add(index, address)?;
                let address = format!("({})", address);
                Reg::set_size(self.data_type().size());
                let result = gen.emit(&format!("\tmov \t{}, {}\n", Reg::current(), address));
                Reg::pop();
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
            Assignment::PtrAssignment { value, address: _ } => value.data_type(),
            Assignment::ArrayAssignment {
                index: _,
                value,
                address: _,
            } => value.data_type(),
        }
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
                if *data_type != expression.data_type()
                    && !expression.data_type().can_convert(data_type.clone())
                {
                    lexer.error(format!(
                        "Cannot assign {:?} to {:?}!",
                        expression.data_type(),
                        data_type
                    ))?;
                }
                Self::VariableAssignment {
                    stack_offset: *stack_offset,
                    expression: expression,
                }
            }
            Expression::Indexing { index, operand } => {
                let expression = Expression::parse(lexer, scope)?;
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
                    Self::PtrAssignment {
                        address: address.clone(),
                        value: expression,
                    }
                }
                _ => lexer
                    .error("can only assing expression to a variable or pointer!".to_string())?,
            },
            _ => lexer.error("Cannot assign expression to non variable!".to_string())?,
        }))
    }
}
