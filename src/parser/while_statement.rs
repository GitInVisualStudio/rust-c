use std::rc::Rc;

use crate::lexer::tokens::Token;

use super::{expression::Expression, statement_list::StatementList, ASTNode, generator::Generator};

#[derive(Debug)]
pub struct WhileStatement {
    condition: Rc<Expression>,
    body: Rc<StatementList>,
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
        Ok(Rc::new(WhileStatement { condition, body }))
    }

    fn generate(&self, gen: &mut super::generator::Generator) -> Result<usize, std::io::Error> {
        let (condition, end) = Generator::generate_clause_names();
        gen.emit_label(&condition)?;
        self.condition.generate(gen)?;
        
        gen.emit_ins("cmp ", "$0", "%eax")?;
        gen.emit(format!("\tje\t\t{}\n", end))?;

        self.body.generate(gen)?;

        gen.emit(format!("\tjmp\t\t{}\n", condition))?;
        gen.emit_label(&end)?;
        
        Ok(0)
    }
}
