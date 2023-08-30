use std::rc::Rc;

use crate::lexer::{tokens::Token, LexerError};

use super::{expression::Expression, variable::DataType, ASTNode, generator::register::Reg};

#[derive(Debug)]
pub struct ArrayExpression {
    data_type: DataType,
    expressinos: Vec<Rc<Expression>>,
    offset: usize,
    base_type: DataType,
}

impl ASTNode for ArrayExpression {
    fn parse(
        lexer: &mut crate::lexer::Lexer,
        scope: &mut super::scope::Scope,
    ) -> Result<Rc<Self>, LexerError>
    where
        Self: Sized,
    {
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
                // if expr.data_type() != last.data_type() {
                //     lexer.error("Array member must share the same datatype!".to_string())?
                // }
            }
            expressions.push(expr);
            if lexer.peek() == Token::COMMA {
                lexer.next();
            }
            scope.add_stack(data_type.size());
        }
        lexer.next();
        let base_type = expressions.first().unwrap().data_type();
        Ok(Rc::new(ArrayExpression {
            data_type: DataType::PTR(Rc::new(base_type.clone())),
            expressinos: expressions,
            offset: scope.stack_size(),
            base_type: base_type
        }))
    }

    fn generate(&self, gen: &mut super::generator::Generator) -> Result<usize, std::io::Error> {
        let mut offset = self.offset;
        for expr in &self.expressinos {
            expr.generate(gen)?;
            gen.mov(Reg::current(), Reg::STACK { offset: offset })?;
            offset -= self.base_type.size();
        }
        Reg::set_size(8);
        gen.lea(Reg::STACK { offset: self.offset }, Reg::current())
    }
}

impl ArrayExpression {
    pub fn data_type(&self) -> DataType {
        self.data_type.clone()
    }
}
