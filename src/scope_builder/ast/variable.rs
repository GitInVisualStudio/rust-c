use super::DataType;

#[derive(Debug, Clone, Copy)]
pub struct Variable<'a> {
    pub(crate) stack_offset: usize,
    pub(crate) data_type: DataType<'a>,
}

impl<'a> Variable<'a> {
    pub fn new(stack_offset: usize, data_type: DataType<'a>) -> Variable<'a> {
        Variable {
            stack_offset,
            data_type,
        }
    }
}
