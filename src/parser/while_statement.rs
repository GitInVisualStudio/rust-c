use std::rc::Rc;

use crate::lexer::tokens::Token;

use super::{expression::Expression, generator::{Generator, register::Reg}, statement_list::StatementList, ASTNode};

#[derive(Debug)]
pub struct WhileStatement {
    condition: Rc<Expression>,
    body: Rc<StatementList>,
    label_index: usize,
}

impl ASTNode for WhileStatement {
    fn parse(
        lexer: &mut crate::lexer::Lexer,
        scope: &mut super::scope::Scope,
    ) -> Result<Rc<Self>, crate::lexer::LexerError>
    where
        Self: Sized,
    {
        lexer.expect_tokens(&[Token::WHILE, Token::LPAREN])?;
        let condition = Expression::parse(lexer, scope)?;
        lexer.expect(Token::RPAREN)?;
        let body = StatementList::parse(lexer, scope)?;
        Ok(Rc::new(WhileStatement {
            condition,
            body,
            label_index: Generator::next_label_index(),
        }))
    }

    fn generate(&self, gen: &mut super::generator::Generator) -> Result<usize, std::io::Error> {
        let (condition, end, _) = Generator::generate_label_names(self.label_index);
        gen.emit_label(&condition)?;
        self.condition.generate(gen)?;

        gen.cmp(Reg::IMMEDIATE(0), Reg::current())?;
        gen.emit(&format!("\tje\t\t{}\n", end))?;

        self.body.generate(gen)?;

        gen.emit(&format!("\tjmp\t\t{}\n", condition))?;
        gen.emit_label(&end)?;

        Ok(0)
    }
}
