use crate::visitor::Visitable;

use super::{resolved_expression::ResolvedExpression, DataType};

#[derive(Debug)]
pub struct ResolvedFunctionCall<'a> {
    pub(crate) name: &'a str,
    pub(crate) parameter: Vec<&'a ResolvedExpression<'a>>,
    pub(crate) return_type: DataType<'a>
}

impl Visitable for ResolvedFunctionCall<'_> {}
