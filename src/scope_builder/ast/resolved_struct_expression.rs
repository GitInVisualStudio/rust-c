use crate::visitor::Visitable;

use super::{resolved_expression::ResolvedExpression, DataType};

#[derive(Debug)]
pub struct ResolvedStructExpression<'a> {
    pub(crate) fields: Vec<(&'a str, &'a ResolvedExpression<'a>)>,
    pub(crate) data_type: DataType<'a>
}

impl Visitable for ResolvedStructExpression<'_> {}
