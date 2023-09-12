use derive_getters::Getters;

use crate::{
    lexer::{tokens::Token, LexerError},
    parser::{Parse, Parser},
};

use super::{
    assignment::Assignment, data_type::DataType, expression::Expression, variable::Variable,
    ASTNode,
};

#[derive(Debug, Getters)]
pub struct StructExpression {
    assignments: Vec<Assignment>,
    offset: usize,
    data_type: DataType,
}

impl ASTNode for StructExpression {}

impl Parse<StructExpression> for Parser<'_> {
    fn parse(&mut self) -> Result<StructExpression, LexerError> {
        let mut fields: Vec<Variable> = Vec::new();
        let mut expressions: Vec<Expression> = Vec::new();
        let mut offset = 0;
        while self.peek() != Token::RCURL {
            self.expect(Token::DOT)?;
            let name = self.expect(Token::IDENT)?.to_string();
            self.expect(Token::ASSIGN)?;
            let expression: Expression = self.parse()?;
            let var = Variable::new(&name, expression.data_type(), offset);

            offset += expression.data_type().size();
            fields.push(var);
            expressions.push(expression);

            if self.peek() != Token::RCURL {
                self.expect(Token::COMMA)?;
            }
        }
        self.scope.add_stack(offset);

        let mut assignments: Vec<_> = Vec::new();
        let mut offset = self.scope.stack_size();
        for expression in expressions {
            let size = expression.data_type().size();
            assignments.push(Assignment::VariableAssignment {
                stack_offset: offset,
                expression: expression,
            });
            offset -= size;
        }

        self.expect(Token::RCURL)?;
        for struct_ in self.scope.get_structs() {
            if struct_.fields_equal(&fields) {
                return Ok(StructExpression {
                    assignments: assignments,
                    offset: self.scope.stack_size(),
                    data_type: DataType::STRUCT(struct_.clone()),
                });
            }
        }
        self.error("Struct expressions does not equal any struct!".to_string())
    }
}

