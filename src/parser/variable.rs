#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    INT,
}

#[derive(Debug)]
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

    pub fn size(&self) -> usize {
        match self.data_type {
            DataType::INT => 4,
        }
    }

    pub fn offset(&self) -> usize {
        self.offset + 4
    }

    pub fn data_type(&self) -> DataType {
        self.data_type.clone()
    }
}
