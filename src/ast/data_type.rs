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

#[derive(Debug, Clone, PartialEq)]
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
            (DataType::STRUCT(x), DataType::STRUCT(y)) if *x == y => true,
            (DataType::PTR(x), DataType::PTR(y)) if *x.as_ref() == *y.as_ref() => true,
            (DataType::PTR(x), DataType::PTR(_)) if *x.as_ref() == DataType::VOID => true,
            (DataType::PTR(_), DataType::INT) => true,
            (_, DataType::STRUCT(_)) => false,
            (DataType::STRUCT(_), _) => false,
            (DataType::PTR(_), _) => false,
            _ => true,
        }
    }

    pub fn can_operate(&self, to: DataType) -> bool {
        match (self, to) {
            (DataType::VOID, DataType::VOID) => false,
            (DataType::VOID, _) => false,
            (_, DataType::VOID) => false,
            (_, DataType::STRUCT(_)) => false,
            (DataType::STRUCT(_), _) => false,
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

    pub fn fields_equal(&self, other: &Vec<Variable>) -> bool {
        &self.fields == other
    }
}
