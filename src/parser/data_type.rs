use std::{io::Error, rc::Rc};

use super::{
    generator::{register::Reg, Generator},
    variable::Variable,
};

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

    fn mov_bytes(
        gen: &mut Generator,
        from: Reg,
        to: Reg,
        bytes_to_copy: usize,
        total_size: usize,
    ) -> Result<usize, Error> {
        match bytes_to_copy {
            1 => {
                let bytes = 1;
                Reg::set_size(bytes);
                let current = format!("{}", Reg::current());
                Reg::set_size(8);
                gen.emit(&format!(
                    "\tmovb\t{}({}), {}\n",
                    total_size - bytes_to_copy,
                    from,
                    current
                ))?;
                gen.emit(&format!("\tmov \t{}, ({})\n", current, to,))?;
                gen.add(Reg::IMMEDIATE(bytes as i64), to)?;
                Ok(bytes_to_copy - bytes)
            }
            2..=3 => {
                let bytes = 2;
                Reg::set_size(bytes);
                let current = format!("{}", Reg::current());
                Reg::set_size(8);
                gen.emit(&format!(
                    "\tmovw\t{}({}), {}\n",
                    total_size - bytes_to_copy,
                    from,
                    current
                ))?;
                gen.emit(&format!("\tmov \t{}, ({})\n", current, to,))?;
                gen.add(Reg::IMMEDIATE(bytes as i64), to)?;
                Ok(bytes_to_copy - bytes)
            }
            4..=7 => {
                let bytes = 4;
                Reg::set_size(bytes);
                let current = format!("{}", Reg::current());
                Reg::set_size(8);
                gen.emit(&format!(
                    "\tmov \t{}({}), {}\n",
                    total_size - bytes_to_copy,
                    from,
                    current
                ))?;
                gen.emit(&format!("\tmov \t{}, ({})\n", current, to,))?;
                gen.add(Reg::IMMEDIATE(bytes as i64), to)?;
                Ok(bytes_to_copy - bytes)
            }
            _ => {
                let bytes = 8;
                Reg::set_size(bytes);
                let current = format!("{}", Reg::current());
                Reg::set_size(8);
                gen.emit(&format!(
                    "\tmovq\t{}({}), {}\n",
                    total_size - bytes_to_copy,
                    from,
                    current
                ))?;
                gen.emit(&format!("\tmov \t{}, ({})\n", current, to,))?;
                gen.add(Reg::IMMEDIATE(bytes as i64), to)?;
                Ok(bytes_to_copy - bytes)
            }
        }
    }

    pub fn mov(
        gen: &mut Generator,
        from: Reg,
        to: Reg,
        data_type: DataType,
    ) -> Result<usize, Error> {
        let size = data_type.size();
        let mut bytes_to_copy = size;
        while bytes_to_copy > 0 {
            bytes_to_copy = Self::mov_bytes(gen, from, to, bytes_to_copy, size)?;
        }
        Ok(0)
    }
}
