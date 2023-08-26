use std::rc::Rc;

use crate::lexer::{tokens::Token, LexerError};

use super::{
    expression::Expression, generator::Generator, scope::Scope, statement_list::StatementList,
    ASTNode,
};

#[derive(Debug)]
pub struct IfStatement {
    statements: Rc<StatementList>,
    condition: Rc<Expression>,
    else_part: Option<Rc<dyn ASTNode>>,
}

impl ASTNode for IfStatement {
    fn parse(lexer: &mut crate::lexer::Lexer, scope: &mut Scope) -> Result<Rc<Self>, LexerError>
    where
        Self: Sized,
    {
        lexer.expect(Token::IF)?;
        lexer.expect(Token::LPAREN)?;
        let condition = Expression::parse(lexer, scope)?;
        lexer.expect(Token::RPAREN)?;
        let statements = StatementList::parse(lexer, scope)?;
        let mut else_part: Option<Rc<dyn ASTNode>> = None;
        if lexer.peek() == Token::ELSE {
            lexer.next();
            else_part = match lexer.peek() {
                Token::IF => Some(IfStatement::parse(lexer, scope)?),
                _ => Some(StatementList::parse(lexer, scope)?)
            };
        }
        Ok(Rc::new(IfStatement {
            statements: statements,
            condition: condition,
            else_part: else_part,
        }))
    }

    fn generate(&self, gen: &mut Generator) -> Result<usize, std::io::Error> {
        self.condition.generate(gen)?;
        let (else_part, end) = Generator::generate_clause_names();
        gen.emit_ins("cmpl", "$0", "%eax")?;
        gen.emit(format!("\tje\t\t{}\n", else_part))?;

        self.statements.generate(gen)?;

        gen.emit(format!("\tjmp \t{}\n", end))?;
        gen.emit_label(&else_part)?;

        if let Some(else_part) = &self.else_part {
            else_part.generate(gen)?;
        }
        gen.emit_label(&end)?;
        Ok(0)
    }
}
