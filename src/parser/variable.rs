use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    LONG,
    INT,
    CHAR,
    VOID,
    PTR(Rc<DataType>),
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

    pub fn offset(&self) -> usize {
        self.offset + self.data_type.size()
    }

    pub fn data_type(&self) -> DataType {
        self.data_type.clone()
    }
}

impl DataType {
    pub fn size(&self) -> usize {
        match self {
            DataType::INT => 4,
            DataType::CHAR => 1,
            DataType::LONG => 8,
            DataType::PTR(_) => 8,
            DataType::VOID => 0,
        }
    }

    pub fn can_convert(&self, to: DataType) -> bool {
        match (self, to) {
            _ => true,
        }
    }

    pub fn can_operate(&self, to: DataType) -> bool {
        match (self, to) {
            (DataType::VOID, DataType::VOID) => false,
            (DataType::VOID, _) => false,
            (_, DataType::VOID) => false,
            _ => true,
        }
    }
}
