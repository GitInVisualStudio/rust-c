use std::rc::Rc;

use crate::lexer::tokens::Token;

use super::{
    assignment::Assignment, data_type::DataType, expression::Expression, generator::register::Reg,
    variable::Variable, ASTNode,
};

#[derive(Debug)]
pub struct SturctExpression {
    assignments: Vec<Assignment>,
    offset: usize,
    data_type: DataType,
}

impl ASTNode for SturctExpression {
    fn parse(
        lexer: &mut crate::lexer::Lexer,
        scope: &mut super::scope::Scope,
    ) -> Result<Rc<Self>, crate::lexer::LexerError>
    where
        Self: Sized,
    {
        let mut fields: Vec<Variable> = Vec::new();
        let mut assignments: Vec<Assignment> = Vec::new();
        let mut offset = 0;
        while lexer.peek() != Token::RCURL {
            lexer.expect(Token::DOT)?;
            let name = lexer.expect(Token::IDENT)?.to_string();
            lexer.expect(Token::ASSIGN)?;
            let expression = Expression::parse(lexer, scope)?;
            let var = Variable::new(&name, expression.data_type(), offset);

            offset += expression.data_type().size();
            fields.push(var);
            assignments.push(Assignment::VariableAssignment {
                stack_offset: scope.stack_size() + offset,
                expression: expression,
            });

            if lexer.peek() != Token::RCURL {
                lexer.expect(Token::COMMA)?;
            }
        }
        lexer.expect(Token::RCURL)?;
        scope.add_stack(offset);
        for struct_ in scope.get_structs() {
            if struct_.fields_equal(&fields) {
                return Ok(Rc::new(SturctExpression {
                    assignments: assignments,
                    offset: scope.stack_size(),
                    data_type: DataType::STRUCT(struct_.clone()),
                }));
            }
        }
        lexer.error("Struct expressions does not equal any struct!".to_string())
    }

    fn generate(&self, gen: &mut super::generator::Generator) -> Result<usize, std::io::Error> {
        for assignment in &self.assignments {
            assignment.generate(gen)?;
        }
        Reg::set_size(8);
        gen.lea(
            Reg::STACK {
                offset: self.offset,
            },
            Reg::current(),
        )
    }
}

impl SturctExpression {
    pub fn data_type(&self) -> DataType {
        self.data_type.clone()
    }
}
