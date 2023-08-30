use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    LONG,
    INT,
    CHAR,
    PTR(Rc<DataType>)
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
            DataType::PTR(_) => 8
        }
    }

    pub fn can_convert(&self, other: DataType) -> bool {
        match (self, other) {
            (_, DataType::PTR(_)) => false,
            (DataType::PTR(_), _) => false,
            _ => true
        }
    }

    pub fn can_operate(&self, other: DataType) -> bool {
        match (self, other) {
            _ => true
        }        
    }
}