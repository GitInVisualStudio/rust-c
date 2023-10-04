use crate::visitor::Visitable;

use super::resolved_statement::ResolvedStatement;

#[derive(Debug)]
pub struct ResolvedCompound<'a> {
    pub(crate) statements: Vec<&'a ResolvedStatement<'a>>,
}

impl Visitable for ResolvedCompound<'_> {}
