use crate::visitor::Visitable;

use super::{resolved_expression::ResolvedExpression, resolved_compound::ResolvedCompound};

#[derive(Debug)]
pub enum ResolvedElsePart<'a> {
    IfStatement(&'a ResolvedIfStatement<'a>),
    Compound(&'a ResolvedCompound<'a>),
    None,
}

#[derive(Debug)]
pub struct ResolvedIfStatement<'a> {
    pub(crate) statements: &'a ResolvedCompound<'a>,
    pub(crate) condition: &'a ResolvedExpression<'a>,
    pub(crate) else_part: ResolvedElsePart<'a>,
}

impl Visitable for ResolvedIfStatement<'_> {}
