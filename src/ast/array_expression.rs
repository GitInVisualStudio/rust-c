use std::rc::Rc;

use crate::{
    lexer::{tokens::Token, LexerError},
    parser::{Parse, Parser},
};

use super::{data_type::DataType, expression::Expression, ASTNode};

#[derive(Debug)]
pub enum ArrayExpression {
    StackArray {
        data_type: DataType,
        expressinos: Vec<Expression>,
        offset: usize,
        base_type: DataType,
    },
    StringLiteral {
        label: usize,
        string: String,
    },
}

impl ASTNode for ArrayExpression {}

impl Parse<ArrayExpression> for Parser<'_> {
    fn parse(&mut self) -> Result<ArrayExpression, LexerError> {
        Ok(match self.peek() {
            Token::STRINGLIT => {
                let label_index = self.next_label_index();
                let string = self.expect(Token::STRINGLIT)?;
                ArrayExpression::StringLiteral {
                    label: label_index,
                    string: string.to_string(),
                }
            }
            _ => {
                let mut expressions: Vec<Expression> = Vec::new();
                if self.peek() == Token::RCURL {
                    self.error("Cannot create empty array".to_string())?;
                }
                while self.peek() != Token::RCURL {
                    let expr: Expression = self.parse()?;
                    let last = expressions.last();
                    let data_type = expr.data_type();
                    if let Some(last) = last {
                        if expr.data_type() != last.data_type()
                            && !expr.data_type().can_convert(last.data_type())
                        {
                            self.error("Array member must share the same datatype!".to_string())?
                        }
                    }
                    expressions.push(expr);
                    if self.peek() == Token::COMMA {
                        self.next();
                    }
                    self.scope.add_stack(data_type.size())
                }
                self.next();
                let base_type = expressions.first().unwrap().data_type();
                ArrayExpression::StackArray {
                    data_type: DataType::PTR(Rc::new(base_type.clone())),
                    expressinos: expressions,
                    offset: self.scope.stack_size(),
                    base_type: base_type,
                }
            }
        })
    }
}

impl ArrayExpression {
    pub fn data_type(&self) -> DataType {
        match self {
            ArrayExpression::StackArray {
                data_type,
                expressinos: _,
                offset: _,
                base_type: _,
            } => data_type.clone(),
            ArrayExpression::StringLiteral {
                label: _,
                string: _,
            } => DataType::PTR(Rc::new(DataType::CHAR)),
        }
    }
}
