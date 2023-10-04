use crate::visitor::Visitable;

use super::{
    resolved_compound::ResolvedCompound, resolved_for::ResolvedForStatement,
    resolved_if::ResolvedIfStatement, resolved_while::ResolvedWhileStatement,
    resolved_expression::ResolvedExpression,
};

#[derive(Debug)]
pub enum ResolvedStatement<'a> {
    Return(Option<&'a ResolvedExpression<'a>>),
    SingleExpression(&'a ResolvedExpression<'a>),
    Compound(&'a ResolvedCompound<'a>),
    IfStatement(&'a ResolvedIfStatement<'a>),
    ForStatement(&'a ResolvedForStatement<'a>),
    WhileStatement(&'a ResolvedWhileStatement<'a>),
    VariableDeclaration {
        stack_offset: usize,
        assignment: Option<&'a ResolvedExpression<'a>>,
    },
    //TODO: have to select the right label index
    Conitnue(i32),
    Break(i32),
    Empty,
}

impl Visitable for ResolvedStatement<'_> {}
