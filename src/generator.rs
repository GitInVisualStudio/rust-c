pub mod register;

use std::{
    fs::File,
    io::{BufWriter, Error, Write},
    rc::Rc,
    sync::atomic::AtomicUsize,
};

use crate::ast::{
    array_expression::ArrayExpression,
    assignment::Assignment,
    data_type::DataType,
    expression::{BinaryOps, Expression, UnaryOps},
    for_statement::ForStatement,
    function::Function,
    function_call::FunctionCall,
    if_statement::{ElsePart, IfStatement},
    program::{Decalrations, Program},
    statement::Statement,
    statement_list::StatementList,
    struct_expression::StructExpression,
    while_statement::WhileStatement,
    ASTNode, Visitor,
};

use self::register::Reg;

pub static CLAUSE_COUNT: AtomicUsize = AtomicUsize::new(0);
pub struct Generator {
    writer: BufWriter<File>,
}

impl Generator {
    pub fn new(file_name: &str) -> Result<Generator, Error> {
        let file = File::create(file_name)?;
        Ok(Generator {
            writer: BufWriter::new(file),
        })
    }

    fn accept<'a, T, R>(&mut self, visitor: &'a T) -> R
    where
        T: ASTNode,
        Self: Visitor<&'a T, R>,
    {
        visitor.accept(self)
    }

    pub fn generate(&mut self, program: &Program) -> Result<usize, Error> {
        self.accept(program)
    }

    pub fn emit(&mut self, string: &str) -> Result<usize, Error> {
        self.writer.write(string.as_bytes())
    }

    pub fn emit_ins(&mut self, ins: &str, from: Reg, to: Reg) -> Result<usize, Error> {
        self.emit(&format!("\t{}\t{}, {}\n", ins, from, to))
    }

    pub fn emit_sins(&mut self, ins: &str, reg: Reg) -> Result<usize, Error> {
        self.emit(&format!("\t{}\t{}\n", ins, reg))
    }

    pub fn mov(&mut self, from: Reg, to: Reg) -> Result<usize, Error> {
        self.emit_ins("mov ", from, to)
    }

    pub fn add(&mut self, from: Reg, to: Reg) -> Result<usize, Error> {
        self.emit_ins("add ", from, to)
    }

    pub fn sub(&mut self, from: Reg, to: Reg) -> Result<usize, Error> {
        self.emit_ins("sub ", from, to)
    }

    pub fn mul(&mut self, from: Reg, to: Reg) -> Result<usize, Error> {
        self.emit_ins("imul", from, to)
    }

    pub fn cmp(&mut self, from: Reg, to: Reg) -> Result<usize, Error> {
        self.emit_ins("cmp ", from, to)
    }

    pub fn cdq(&mut self) -> Result<usize, Error> {
        self.emit("\tcdq\n")
    }

    pub fn jmp(&mut self, label: &str) -> Result<usize, Error> {
        self.emit(&format!("\tjmp\t\t{}\n", label))
    }

    pub fn jne(&mut self, label: &str) -> Result<usize, Error> {
        self.emit(&format!("\tjne\t\t{}\n", label))
    }

    pub fn je(&mut self, label: &str) -> Result<usize, Error> {
        self.emit(&format!("\tje\t\t{}\n", label))
    }

    pub fn gen_cmp(&mut self, ins: &str, from: Reg, to: Reg) -> Result<usize, Error> {
        self.cmp(from, to)?;
        self.mov(Reg::IMMEDIATE(0), to)?;
        let prev = Reg::set_size(1);
        let result = self.emit_sins(ins, to);
        Reg::set_size(prev);
        result
    }

    pub fn lea(&mut self, from: Reg, to: Reg) -> Result<usize, Error> {
        Reg::set_size(8);
        self.emit_ins("lea ", from, to)
    }

    pub fn push_stack(&mut self, size: usize) -> Result<usize, Error> {
        self.emit(&format!(
            "\tpush\t%rbp\n\tmov \t%rsp, %rbp\n\tsub \t${}, %rsp\n",
            (size / 16 + 1) * 16
        ))
    }

    pub fn pop_stack(&mut self) -> Result<usize, Error> {
        self.emit("\tleave\n")
    }

    pub fn emit_label(&mut self, label: &str) -> Result<usize, Error> {
        self.emit(&format!("{}:\n", label))
    }

    pub fn call(&mut self, label: &str) -> Result<usize, Error> {
        self.emit(&format!("\tcall \t{}\n", label))
    }

    pub fn ret(&mut self) -> Result<usize, Error> {
        self.emit("\tret\n")
    }

    pub fn emit_string(&mut self, label: usize, string: &str) -> Result<usize, Error> {
        self.emit(&format!(
            "    .section   .rodata
.LC{}:
    .string	{}
    .text
",
            label, string
        ))
    }

    pub fn generate_clause_names() -> (String, String) {
        let clause_count = CLAUSE_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        (
            format!("_clause{}", clause_count),
            format!("_end{}", clause_count),
        )
    }

    pub fn generate_label_names(label_index: usize) -> (String, String, String) {
        (
            format!("_label{}", label_index),
            format!("_labelend{}", label_index),
            format!("_expression{}", label_index),
        )
    }

    fn mov_bytes(&mut self, from: Reg, to: Reg, bytes: usize) -> Result<usize, Error> {
        let total_size = bytes;
        let mut bytes_to_copy = total_size;
        let from = from.as_address();
        while bytes_to_copy > 0 {
            match bytes_to_copy {
                1 => {
                    let bytes = 1;
                    Reg::set_size(bytes);
                    self.mov(from.offset(total_size - bytes_to_copy), Reg::current())?;
                    self.mov(Reg::current(), to.as_address())?;
                    self.add(Reg::IMMEDIATE(bytes as i64), to)?;
                    bytes_to_copy -= bytes
                }
                2..=3 => {
                    let bytes = 2;
                    Reg::set_size(bytes);
                    self.mov(from.offset(total_size - bytes_to_copy), Reg::current())?;
                    self.mov(Reg::current(), to.as_address())?;
                    self.add(Reg::IMMEDIATE(bytes as i64), to)?;
                    bytes_to_copy -= bytes
                }
                4..=7 => {
                    let bytes = 4;
                    Reg::set_size(bytes);
                    self.mov(from.offset(total_size - bytes_to_copy), Reg::current())?;
                    self.mov(Reg::current(), to.as_address())?;
                    self.add(Reg::IMMEDIATE(bytes as i64), to)?;
                    bytes_to_copy -= bytes
                }
                _ => {
                    let bytes = 8;
                    Reg::set_size(bytes);
                    self.mov(from.offset(total_size - bytes_to_copy), Reg::current())?;
                    self.mov(Reg::current(), to.as_address())?;
                    self.add(Reg::IMMEDIATE(bytes as i64), to)?;
                    bytes_to_copy -= bytes
                }
            }
        }
        Ok(0)
    }

    fn generate_and_or(&mut self, expression: &Expression) -> Result<usize, Error> {
        if let Expression::BinaryExpression {
            first,
            second,
            operation,
        } = expression
        {
            let first_reg = Reg::current();
            self.accept(first.as_ref())?;

            let (second_expression_label, end_label) = Generator::generate_clause_names();
            match *operation {
                BinaryOps::AND => {
                    self.cmp(Reg::IMMEDIATE(0), first_reg)?;
                    self.jne(&second_expression_label)?;
                    self.jmp(&end_label)
                }
                BinaryOps::OR => {
                    self.cmp(Reg::IMMEDIATE(0), first_reg)?;
                    self.je(&second_expression_label)?;
                    self.jmp(&end_label)
                }
                _ => panic!("Wrong operation for boolean comparision!"),
            }?;
            self.emit_label(&second_expression_label)?;
            self.accept(second.as_ref())?;
            let second_reg = Reg::current();

            self.cmp(Reg::IMMEDIATE(0), second_reg)?;
            self.mov(Reg::IMMEDIATE(1), second_reg)?;

            let prev = Reg::set_size(1);
            self.emit_sins("setne", second_reg)?;
            Reg::set_size(prev);

            return self.emit_label(&end_label);
        }
        panic!("this should not happen!");
    }
}

impl Visitor<&Program, Result<usize, Error>> for Generator {
    fn visit(&mut self, visitor: &Program) -> Result<usize, Error> {
        self.emit(
            &"
    .text
    .globl	main
    .type	main, @function
"
            .to_string(),
        )?;
        for x in visitor.declarations() {
            match x {
                Decalrations::Statement(statement) => self.accept(statement),
                Decalrations::Function(x) => self.accept(x),
            }?;
        }
        Ok(0)
    }
}

impl Visitor<&Statement, Result<usize, Error>> for Generator {
    fn visit(&mut self, visitor: &Statement) -> Result<usize, Error> {
        match visitor {
            Statement::Return(expression) => {
                if expression.is_some() {
                    expression.as_ref().unwrap().accept(self)?;
                }
                self.pop_stack()?;
                self.mov(Reg::current(), Reg::RAX)?;
                self.ret()
            }
            Statement::IfStatement(statement) => self.accept(statement),
            Statement::SingleExpression(expression) => self.accept(expression),
            Statement::ForStatement(statement) => self.accept(statement),
            Statement::WhileStatement(while_statement) => self.accept(while_statement),
            Statement::StatementList(list) => self.accept(list),
            // dont need to generate TypeDefinitions
            Statement::TypeDefinition(_) => Ok(0),
            Statement::Empty => Ok(0),
            Statement::Continue { label_index } => {
                let (_, _, condition) = Generator::generate_label_names(*label_index);
                self.jmp(&condition)
            }
            Statement::Break { label_index } => {
                let (_, end, _) = Generator::generate_label_names(*label_index);
                self.jmp(&end)
            }
            Statement::VariableDeclaration {
                variable: _,
                expression,
            } => {
                if expression.is_some() {
                    expression.as_ref().unwrap().accept(self)?;
                }
                Ok(0)
            }
        }
    }
}

impl Visitor<&Rc<Function>, Result<usize, Error>> for Generator {
    fn visit(&mut self, visitor: &Rc<Function>) -> Result<usize, Error> {
        if visitor.statements().is_none() {
            return Ok(0);
        }
        self.emit_label(&visitor.name())?;
        self.push_stack(*visitor.stack_size())?;

        //push parameter onto the local stack
        for (index, parameter) in visitor.parameter().iter().enumerate() {
            match parameter.data_type() {
                DataType::STRUCT(_) => {
                    let to = Reg::push();
                    self.lea(
                        Reg::STACK {
                            offset: parameter.offset(),
                        },
                        to,
                    )?;
                    let result = self.mov_bytes(
                        Reg::get_parameter_index(index),
                        to,
                        parameter.data_type().size(),
                    );
                    Reg::pop();
                    result
                }
                _ => {
                    Reg::set_size(parameter.data_type().size());
                    self.mov(
                        Reg::get_parameter_index(index),
                        Reg::STACK {
                            offset: parameter.offset(),
                        },
                    )
                }
            }?;
        }

        visitor.statements().as_ref().unwrap().accept(self)?;

        if *visitor.return_type() == DataType::VOID {
            self.pop_stack()?;
            self.ret()?;
        }
        Ok(0)
    }
}

impl Visitor<&Expression, Result<usize, Error>> for Generator {
    fn visit(&mut self, visitor: &Expression) -> Result<usize, Error> {
        match visitor {
            Expression::IntLiteral(value) => {
                Reg::set_size(4);
                self.mov(Reg::IMMEDIATE(*value as i64), Reg::current())
            }
            Expression::LongLiteral(value) => {
                Reg::set_size(8);
                self.mov(Reg::IMMEDIATE(*value), Reg::current())
            }
            Expression::CharLiteral(value) => {
                Reg::set_size(1);
                self.mov(Reg::IMMEDIATE(*value as i64), Reg::current())
            }
            Expression::NamedVariable {
                stack_offset,
                data_type,
            } => match data_type {
                DataType::STRUCT(_) => self.lea(
                    Reg::STACK {
                        offset: *stack_offset,
                    },
                    Reg::current(),
                ),
                x => {
                    Reg::set_size(x.size());
                    self.mov(
                        Reg::STACK {
                            offset: *stack_offset,
                        },
                        Reg::current(),
                    )
                }
            },
            Expression::Unary {
                expression,
                operation,
            } => match operation {
                UnaryOps::NEG => {
                    self.accept(expression.as_ref())?;
                    let reg = Reg::current();
                    self.emit_sins("neg", reg)
                }
                UnaryOps::COMPLEMENT => {
                    self.accept(expression.as_ref())?;
                    let reg = Reg::current();
                    self.emit_sins("not", reg)
                }
                UnaryOps::LOGNEG => {
                    self.accept(expression.as_ref())?;
                    let reg = Reg::current();
                    self.cmp(Reg::IMMEDIATE(0), reg)?;
                    self.mov(Reg::IMMEDIATE(0), reg)?;
                    let prev = Reg::set_size(1);
                    let result = self.emit_sins("sete", reg);
                    Reg::set_size(prev);
                    result
                }
                UnaryOps::REF => match expression.as_ref() {
                    Expression::NamedVariable {
                        stack_offset,
                        data_type: _,
                    } => self.lea(
                        Reg::STACK {
                            offset: *stack_offset,
                        },
                        Reg::current(),
                    ),
                    _ => panic!("should not happen!"),
                },
                UnaryOps::DEREF => {
                    let base_data_type = match expression.data_type() {
                        DataType::PTR(x) => x,
                        _ => panic!("cannot get base data-type from index"),
                    };
                    match base_data_type.as_ref() {
                        DataType::STRUCT(_) => self.accept(expression.as_ref()),
                        _ => {
                            self.accept(expression.as_ref())?;
                            let address = Reg::current().as_address();
                            Reg::set_size(base_data_type.size());
                            self.mov(address, Reg::current())
                        }
                    }
                }
                UnaryOps::CAST(_) => self.accept(expression.as_ref()),
            },
            Expression::BinaryExpression {
                first,
                second,
                operation,
            } => {
                if *operation == BinaryOps::AND || *operation == BinaryOps::OR {
                    return self.generate_and_or(visitor);
                }

                let first_reg = Reg::current();
                self.accept(first.as_ref())?;
                Reg::push();
                self.accept(second.as_ref())?;
                let second_reg = Reg::pop();
                Reg::set_size(visitor.data_type().size());
                match *operation {
                    BinaryOps::ADD => self.add(second_reg, first_reg)?,
                    BinaryOps::SUB => self.sub(second_reg, first_reg)?,
                    BinaryOps::MUL => self.mul(second_reg, first_reg)?,
                    BinaryOps::DIV => {
                        self.mov(second_reg, Reg::RBX)?;
                        self.mov(first_reg, Reg::RAX)?;
                        self.cdq()?;
                        self.emit_sins("idiv", Reg::RBX)?;
                        self.mov(Reg::RAX, Reg::current())?
                    }
                    BinaryOps::MOD => {
                        self.mov(second_reg, Reg::RBX)?;
                        self.mov(first_reg, Reg::RAX)?;
                        self.cdq()?;
                        self.emit_sins("idiv", Reg::RBX)?;
                        self.mov(Reg::RDX, Reg::current())?
                    }
                    BinaryOps::EQ => self.gen_cmp("sete", second_reg, first_reg)?,
                    BinaryOps::NE => self.gen_cmp("setne", second_reg, first_reg)?,
                    BinaryOps::LT => self.gen_cmp("setl", second_reg, first_reg)?,
                    BinaryOps::GT => self.gen_cmp("setg", second_reg, first_reg)?,
                    BinaryOps::LE => self.gen_cmp("setle", second_reg, first_reg)?,
                    BinaryOps::GE => self.gen_cmp("setge", second_reg, first_reg)?,
                    _ => panic!("Something went wrong"),
                };
                Ok(0)
            }
            Expression::FunctionCall(call) => self.accept(call),
            Expression::ArrayExpression(arr) => self.accept(arr),
            Expression::Indexing { index, operand } => {
                let base_data_type = match operand.data_type() {
                    DataType::PTR(x) => x,
                    _ => panic!("cannot get base data-type from index"),
                };

                self.accept(index.as_ref())?;
                let index = Reg::push();
                self.accept(operand.as_ref())?;
                let address = Reg::pop();

                Reg::set_size(8);
                self.mul(Reg::IMMEDIATE(visitor.data_type().size() as i64), index)?;
                self.add(index, address)?;

                match base_data_type.as_ref() {
                    DataType::STRUCT(_) => {
                        Reg::set_size(base_data_type.size());
                        self.mov(address, Reg::current())
                    }
                    _ => {
                        let address = address.as_address();
                        Reg::set_size(base_data_type.size());
                        self.mov(address, Reg::current())
                    }
                }
            }
            Expression::Assignment(assignment) => self.accept(assignment.as_ref()),
            // dont have to generateo
            Expression::TypeExpression(_) => Ok(0),
            Expression::FieldAccess {
                offset,
                data_type,
                operand,
            } => {
                let offset = *offset;
                match data_type {
                    DataType::STRUCT(_) => {
                        self.accept(operand.as_ref())?;
                        self.add(Reg::IMMEDIATE(offset as i64), Reg::current())
                    }
                    data_type => {
                        self.accept(operand.as_ref())?;
                        Reg::set_size(data_type.size());
                        self.mov(Reg::current().as_address().offset(offset), Reg::current())
                    }
                }
            }
            Expression::StructExpresion(expr) => self.accept(expr),
        }
    }
}

impl Visitor<&IfStatement, Result<usize, Error>> for Generator {
    fn visit(&mut self, visitor: &IfStatement) -> Result<usize, Error> {
        self.accept(visitor.condition())?;

        let (else_part, end) = Generator::generate_clause_names();
        self.cmp(Reg::IMMEDIATE(0), Reg::current())?;

        self.je(&else_part)?;

        self.accept(visitor.statements())?;

        self.jmp(&end)?;
        self.emit_label(&else_part)?;

        match visitor.else_part().as_ref() {
            ElsePart::IfStatement(x) => self.accept(x),
            ElsePart::StatementList(x) => self.accept(x),
            ElsePart::None => Ok(0),
        }?;
        self.emit_label(&end)
    }
}

impl Visitor<&ForStatement, Result<usize, Error>> for Generator {
    fn visit(&mut self, visitor: &ForStatement) -> Result<usize, Error> {
        let (body, end, post) = Self::generate_label_names(*visitor.label_index());
        self.accept(visitor.init().as_ref())?;

        self.emit_label(&body)?;

        self.accept(visitor.condition())?;
        self.cmp(Reg::IMMEDIATE(0), Reg::current())?;
        // jump to end of for if the condition is not met anymore
        self.je(&end)?;

        self.accept(visitor.body())?;

        self.emit_label(&post)?;
        if let Some(post) = visitor.post() {
            self.accept(post)?;
        }

        self.jmp(&body)?;
        self.emit_label(&end)
    }
}

impl Visitor<&WhileStatement, Result<usize, Error>> for Generator {
    fn visit(&mut self, visitor: &WhileStatement) -> Result<usize, Error> {
        let (condition, end, _) = Self::generate_label_names(*visitor.label_index());
        self.emit_label(&condition)?;
        self.accept(visitor.condition())?;

        self.cmp(Reg::IMMEDIATE(0), Reg::current())?;
        self.je(&end)?;

        self.accept(visitor.body())?;

        self.jmp(&condition)?;
        self.emit_label(&end)
    }
}

impl Visitor<&StatementList, Result<usize, Error>> for Generator {
    fn visit(&mut self, visitor: &StatementList) -> Result<usize, Error> {
        for s in visitor.statements() {
            self.accept(s)?;
        }
        Ok(0)
    }
}

impl Visitor<&Assignment, Result<usize, Error>> for Generator {
    fn visit(&mut self, visitor: &Assignment) -> Result<usize, Error> {
        match visitor {
            Assignment::VariableAssignment {
                stack_offset,
                expression,
            } => match expression.data_type() {
                DataType::STRUCT(_) => {
                    self.accept(expression)?;
                    let from = Reg::push();
                    let to = Reg::push();
                    self.lea(
                        Reg::STACK {
                            offset: *stack_offset,
                        },
                        to,
                    )?;
                    let result = self.mov_bytes(from, to, expression.data_type().size());
                    Reg::pop();
                    Reg::pop();

                    result
                }
                _ => {
                    Reg::set_size(visitor.data_type().size());
                    self.accept(expression)?;
                    self.mov(
                        Reg::current(),
                        Reg::STACK {
                            offset: *stack_offset,
                        },
                    )
                }
            },
            Assignment::PtrAssignment { value, address } => match value.data_type() {
                DataType::STRUCT(_) => {
                    Reg::set_size(visitor.data_type().size());
                    let data_type = value.data_type();
                    self.accept(address.as_ref())?;
                    let address = Reg::push();
                    self.accept(value)?;
                    let value = Reg::push();

                    let result = self.mov_bytes(value, address, data_type.size());

                    Reg::pop();
                    Reg::pop();
                    result
                }
                _ => {
                    Reg::set_size(visitor.data_type().size());
                    self.accept(address.as_ref())?;
                    let address = Reg::push().as_address();
                    self.accept(value)?;
                    let value = Reg::current();

                    Reg::set_size(visitor.data_type().size());
                    let result = self.mov(value, address);
                    Reg::pop();
                    result
                }
            },
            Assignment::ArrayAssignment {
                index,
                value,
                address,
            } => match value.data_type() {
                DataType::STRUCT(_) => {
                    let data_type = value.data_type();
                    self.accept(value)?;
                    let value = Reg::push();

                    self.accept(address.as_ref())?;
                    let address = Reg::push();

                    self.accept(index.as_ref())?;
                    let index = Reg::current();

                    Reg::set_size(8);
                    self.mul(Reg::IMMEDIATE(visitor.data_type().size() as i64), index)?;
                    self.add(index, address)?;

                    let result = self.mov_bytes(value, address, data_type.size());

                    Reg::pop();
                    Reg::pop();

                    result
                }
                _ => {
                    self.accept(value)?;
                    let value = Reg::push();

                    self.accept(address.as_ref())?;
                    let address = Reg::push();

                    self.accept(index.as_ref())?;
                    let index = Reg::current();

                    self.mul(Reg::IMMEDIATE(visitor.data_type().size() as i64), index)?;
                    Reg::set_size(8);
                    self.add(index, address)?;
                    Reg::set_size(visitor.data_type().size());
                    let result = self.mov(value, address.as_address());

                    Reg::pop();
                    Reg::pop();
                    result
                }
            },
            Assignment::FieldAssignment {
                offset,
                address,
                value,
            } => {
                self.accept(address.as_ref())?;
                let address = Reg::push();
                self.accept(value)?;
                let value = Reg::current();

                // add the offset
                Reg::set_size(8);
                self.add(Reg::IMMEDIATE(*offset as i64), address)?;

                let address = address.as_address();
                Reg::set_size(visitor.data_type().size());
                let result = self.mov(value, address);
                Reg::pop();
                result
            }
        }
    }
}

impl Visitor<&StructExpression, Result<usize, Error>> for Generator {
    fn visit(&mut self, visitor: &StructExpression) -> Result<usize, Error> {
        for assignment in visitor.assignments() {
            self.accept(assignment)?;
        }
        Reg::set_size(8);
        self.lea(
            Reg::STACK {
                offset: *visitor.offset(),
            },
            Reg::current(),
        )
    }
}

impl Visitor<&ArrayExpression, Result<usize, Error>> for Generator {
    fn visit(&mut self, visitor: &ArrayExpression) -> Result<usize, Error> {
        match visitor {
            ArrayExpression::StackArray {
                data_type: _,
                expressinos,
                offset,
                base_type,
            } => {
                let mut start_offset = *offset;
                for expr in expressinos {
                    self.accept(expr)?;
                    self.mov(
                        Reg::current(),
                        Reg::STACK {
                            offset: start_offset,
                        },
                    )?;
                    start_offset -= base_type.size();
                }
                Reg::set_size(8);
                self.lea(Reg::STACK { offset: *offset }, Reg::current())
            }
            ArrayExpression::StringLiteral { label, string } => {
                self.emit_string(*label, string)?;
                Reg::set_size(8);
                self.emit(&format!("\tlea \t.LC{}(%rip), {}\n", label, Reg::current()))
            }
        }
    }
}

impl Visitor<&FunctionCall, Result<usize, Error>> for Generator {
    fn visit(&mut self, visitor: &FunctionCall) -> Result<usize, Error> {
        // store parameter in registers
        for (index, parameter) in visitor.parameter().iter().enumerate() {
            self.accept(parameter)?;
            Reg::set_size(visitor.data_types()[index].size());
            self.mov(Reg::current(), Reg::get_parameter_index(index))?;
        }

        Reg::set_size(8);
        let prev = Reg::current();
        if prev != Reg::R10 {
            while Reg::current() != Reg::R10 {
                self.emit_sins("push", Reg::pop())?;
            }
            self.emit_sins("push", Reg::current())?;
        }
        self.call(visitor.name())?;

        if prev != Reg::R10 {
            Reg::set_size(8);
            while Reg::current() != prev {
                self.emit_sins("pop ", Reg::push())?;
            }
            self.emit_sins("pop ", Reg::current())?;
        }

        Reg::set_size(visitor.return_type().size());
        self.mov(Reg::RAX, Reg::current())
    }
}