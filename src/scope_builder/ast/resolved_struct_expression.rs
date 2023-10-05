use crate::visitor::Visitable;

use super::{DataType, resolved_assignment::ResolvedAssignment};

#[derive(Debug)]
pub struct ResolvedStructExpression<'a> {
    pub(crate) fields: Vec<&'a ResolvedAssignment<'a>>,
    pub(crate) data_type: DataType<'a>,
    pub(crate) stack_offset: usize
}

impl Visitable for ResolvedStructExpression<'_> {}
