use super::data_type::DataType;

#[derive(Debug, PartialEq)]
pub struct Variable {
    name: String,
    data_type: DataType,
    offset: usize,
}

impl Variable {
    pub fn new(name: &str, data_type: DataType, offset: usize) -> Variable {
        Variable {
            name: name.to_owned(),
            data_type: data_type,
            offset: offset,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn offset(&self) -> usize {
        self.offset + self.data_type.size()
    }

    pub fn data_type(&self) -> DataType {
        self.data_type.clone()
    }
}
