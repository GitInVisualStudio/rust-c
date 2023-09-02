use std::rc::Rc;

use crate::{
    lexer::{tokens::Token, LexerError},
    parser::generator::Generator,
};

use super::{expression::Expression, generator::register::Reg, ASTNode, data_type::DataType};

#[derive(Debug)]
pub enum ArrayExpression {
    StackArray {
        data_type: DataType,
        expressinos: Vec<Rc<Expression>>,
        offset: usize,
        base_type: DataType,
    },
    StringLiteral {
        label: usize,
        string: String,
    },
}

impl ASTNode for ArrayExpression {
    fn parse(
        lexer: &mut crate::lexer::Lexer,
        scope: &mut super::scope::Scope,
    ) -> Result<Rc<Self>, LexerError>
    where
        Self: Sized,
    {
        Ok(Rc::new(match lexer.peek() {
            Token::STRINGLIT => {
                let label_index = Generator::next_label_index();
                let string = lexer.expect(Token::STRINGLIT)?;
                ArrayExpression::StringLiteral {
                    label: label_index,
                    string: string.to_string(),
                }
            }
            Token::LCURL => {
                let mut expressions: Vec<Rc<Expression>> = Vec::new();
                lexer.expect(Token::LCURL)?;
                if lexer.peek() == Token::RCURL {
                    lexer.error("Cannot create empty array".to_string())?;
                }
                while lexer.peek() != Token::RCURL {
                    let expr = Expression::parse(lexer, scope)?;
                    let last = expressions.last();
                    let data_type = expr.data_type();
                    if let Some(last) = last {
                        if expr.data_type() != last.data_type()
                            && !expr.data_type().can_convert(last.data_type())
                        {
                            lexer.error("Array member must share the same datatype!".to_string())?
                        }
                    }
                    expressions.push(expr);
                    if lexer.peek() == Token::COMMA {
                        lexer.next();
                    }
                    scope.add_stack(data_type.size());
                }
                lexer.next();
                let base_type = expressions.first().unwrap().data_type();
                ArrayExpression::StackArray {
                    data_type: DataType::PTR(Rc::new(base_type.clone())),
                    expressinos: expressions,
                    offset: scope.stack_size(),
                    base_type: base_type,
                }
            }
            _ => panic!("cannot parse array expression!"),
        }))
    }

    fn generate(&self, gen: &mut super::generator::Generator) -> Result<usize, std::io::Error> {
        match self {
            ArrayExpression::StackArray {
                data_type: _,
                expressinos,
                offset,
                base_type,
            } => {
                let mut start_offset = *offset;
                for expr in expressinos {
                    expr.generate(gen)?;
                    gen.mov(
                        Reg::current(),
                        Reg::STACK {
                            offset: start_offset,
                        },
                    )?;
                    start_offset -= base_type.size();
                }
                Reg::set_size(8);
                gen.lea(Reg::STACK { offset: *offset }, Reg::current())
            }
            ArrayExpression::StringLiteral { label, string } => {
                gen.emit_string(*label, string)?;
                Reg::set_size(8);
                gen.emit(&format!("\tlea \t.LC{}(%rip), {}\n", label, Reg::current()))
            }
        }
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
