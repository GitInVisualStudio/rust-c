use std::rc::Rc;

use super::variable::Variable;

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    LONG,
    INT,
    CHAR,
    VOID,
    PTR(Rc<DataType>),
    STRUCT(Rc<Struct>),
}

#[derive(Debug, PartialEq)]
pub struct Struct {
    name: String,
    fields: Vec<Variable>,
    size: usize,
}

impl DataType {
    pub fn size(&self) -> usize {
        match self {
            DataType::INT => 4,
            DataType::CHAR => 1,
            DataType::LONG => 8,
            DataType::PTR(_) => 8,
            DataType::VOID => 0,
            DataType::STRUCT(x) => x.size(),
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

impl Struct {
    pub fn new(name: String, fields: Vec<Variable>) -> Struct {
        Struct {
            name,
            size: fields.iter().map(|x| x.data_type().size()).sum(),
            fields,
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn get(&self, name: &str) -> Option<&Variable> {
        self.fields.iter().find(|x| x.name() == name)
    }
}
