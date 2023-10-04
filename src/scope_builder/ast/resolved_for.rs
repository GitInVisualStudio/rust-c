use crate::visitor::Visitable;

use super::{resolved_statement::ResolvedStatement, resolved_expression::ResolvedExpression, resolved_compound::ResolvedCompound};

#[derive(Debug)]
pub struct ResolvedForStatement<'a> {
    pub(crate) init: &'a ResolvedStatement<'a>,
    pub(crate) condition: &'a ResolvedExpression<'a>,
    pub(crate) post: Option<&'a ResolvedExpression<'a>>,
    pub(crate) body: &'a ResolvedCompound<'a>,
}

impl Visitable for ResolvedForStatement<'_> {}
