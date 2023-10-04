use crate::visitor::Visitable;

use super::{resolved_expression::ResolvedExpression, resolved_compound::ResolvedCompound};

#[derive(Debug)]
pub struct ResolvedWhileStatement<'a> {
    pub(crate) condition: &'a ResolvedExpression<'a>,
    pub(crate) body: &'a ResolvedCompound<'a>,
}

impl Visitable for ResolvedWhileStatement<'_> {}
