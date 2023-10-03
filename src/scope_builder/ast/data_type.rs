#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataType<'a> {
    LONG,
    INT,
    CHAR,
    VOID,
    PTR(&'a DataType<'a>),
    Struct(&'a Struct<'a>),
    EmptyStruct,
}

#[derive(Debug, PartialEq)]
pub struct Struct<'a> {
    pub(crate) fields: Vec<(&'a str, DataType<'a>)>,
}

impl<'a> DataType<'a> {
    pub fn size(&self) -> usize {
        match self {
            DataType::INT => 4,
            DataType::CHAR => 1,
            DataType::LONG => 8,
            DataType::PTR(_) => 8,
            DataType::Struct(x) => x.fields.iter().map(|x| x.1.size()).sum(),
            DataType::VOID => 0,
            DataType::EmptyStruct => 0,
        }
    }

    pub fn is_number(&self) -> bool {
        match self {
            DataType::INT | DataType::LONG | DataType::CHAR => true,
            _ => false
        }
    }
}

impl<'a> Struct<'a> {
    pub fn new(fields: Vec<(&'a str, DataType<'a>)>) -> Struct<'a> {
        Struct {
            fields: fields,
        }
    }

    pub fn field(&self, name: &'a str) -> Option<DataType<'a>> {
        self.fields
            .iter()
            .find(|(field_name, _)| *field_name == name)
            .map(|x| x.1)
    }
}
