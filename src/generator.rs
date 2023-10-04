pub mod register;

use std::{
    fs::File,
    io::{BufWriter, Error, Write},
};

use crate::{
    parser::ast::{BinaryOps, UnaryOps},
    scope_builder::ast::{
        resolved_array_expression::ResolvedArrayExpression,
        resolved_assignment::ResolvedAssignment,
        resolved_compound::ResolvedCompound,
        resolved_expression::ResolvedExpression,
        resolved_for::ResolvedForStatement,
        resolved_function::ResolvedFunction,
        resolved_function_call::ResolvedFunctionCall,
        resolved_if::{ResolvedElsePart, ResolvedIfStatement},
        resolved_program::ResolvedProgram,
        resolved_statement::ResolvedStatement,
        resolved_struct_expression::ResolvedStructExpression,
        resolved_while::ResolvedWhileStatement,
        DataType,
    },
    visitor::{Visitable, Visitor},
};

use self::register::Reg;

pub struct Generator {
    writer: BufWriter<File>,
    clause_count: i32,
}

impl Generator {
    pub fn new(file_name: &str) -> Result<Generator, Error> {
        let file = File::create(file_name)?;
        Ok(Generator {
            writer: BufWriter::new(file),
            clause_count: 0,
        })
    }

    pub fn generate(&mut self, program: &ResolvedProgram) -> Result<usize, Error> {
        program.accept(self)
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

    pub fn generate_clause_names(&mut self) -> (String, String) {
        self.clause_count += 1;
        let clause_count = self.clause_count;
        (
            format!("_clause{}", clause_count),
            format!("_end{}", clause_count),
        )
    }

    pub fn generate_label_names(label_index: i32) -> (String, String, String) {
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
            let bytes = match bytes_to_copy {
                1 => 1,
                2..=3 => 2,
                4..=7 => 4,
                _ => 8,
            };
            Reg::set_size(bytes);
            self.mov(from.offset(total_size - bytes_to_copy), Reg::current())?;
            self.mov(Reg::current(), to.as_address())?;
            Reg::set_size(8);
            bytes_to_copy -= bytes;
            if bytes_to_copy > 0 {
                self.add(Reg::IMMEDIATE(bytes as i64), to)?;
            }
        }
        Ok(0)
    }

    fn generate_and_or<'a>(&mut self, expression: &ResolvedExpression<'a>) -> Result<usize, Error> {
        if let ResolvedExpression::BinaryExpression {
            lhs,
            rhs,
            operation,
            ..
        } = expression
        {
            let first_reg = Reg::current();
            lhs.accept(self)?;

            let (second_expression_label, end_label) = self.generate_clause_names();
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
            rhs.accept(self)?;
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

impl<'a> Visitor<&ResolvedProgram<'a>, Result<usize, Error>> for Generator {
    fn visit(&mut self, visitor: &ResolvedProgram<'a>) -> Result<usize, Error> {
        self.emit(
            &"
    .text
    .globl	main
    .type	main, @function
"
            .to_string(),
        )?;
        for x in &visitor.functions {
            x.accept(self)?;
        }
        Ok(0)
    }
}

impl<'a> Visitor<&ResolvedCompound<'a>, Result<usize, Error>> for Generator {
    fn visit(&mut self, visitor: &ResolvedCompound<'a>) -> Result<usize, Error> {
        for s in &visitor.statements {
            s.accept(self)?;
        }
        Ok(0)
    }
}

impl<'a> Visitor<&ResolvedStatement<'a>, Result<usize, Error>> for Generator {
    fn visit(&mut self, visitor: &ResolvedStatement<'a>) -> Result<usize, Error> {
        match visitor {
            ResolvedStatement::Return(expression) => {
                if expression.is_some() {
                    expression.as_ref().unwrap().accept(self)?;
                }
                self.pop_stack()?;
                self.mov(Reg::current(), Reg::RAX)?;
                self.ret()
            }
            ResolvedStatement::IfStatement(statement) => statement.accept(self),
            ResolvedStatement::SingleExpression(expression) => expression.accept(self),
            ResolvedStatement::ForStatement(statement) => statement.accept(self),
            ResolvedStatement::WhileStatement(while_statement) => while_statement.accept(self),
            ResolvedStatement::Compound(list) => list.accept(self),
            ResolvedStatement::Empty => Ok(0),
            ResolvedStatement::Continue(label_index) => {
                let (_, _, condition) = Generator::generate_label_names(*label_index);
                self.jmp(&condition)
            }
            ResolvedStatement::Break(label_index) => {
                let (_, end, _) = Generator::generate_label_names(*label_index);
                self.jmp(&end)
            }
            ResolvedStatement::VariableDeclaration { assignment, .. } => {
                if assignment.is_some() {
                    assignment.as_ref().unwrap().accept(self)?;
                }
                Ok(0)
            }
        }
    }
}

impl<'a> Visitor<&ResolvedFunction<'a>, Result<usize, Error>> for Generator {
    fn visit(&mut self, visitor: &ResolvedFunction<'a>) -> Result<usize, Error> {
        if visitor.statements.is_none() {
            return Ok(0);
        }
        self.emit_label(visitor.name)?;
        let parameter_stack_size = visitor.parameter.iter().map(|(t, _)| t.size()).sum();
        self.push_stack(parameter_stack_size)?;

        //push parameter onto the local stack
        let mut offset = 0;
        for (index, (parameter, _)) in visitor.parameter.iter().enumerate() {
            offset += parameter.size();
            match parameter {
                DataType::Struct(_) => {
                    let to = Reg::push();
                    self.lea(Reg::STACK { offset: offset }, to)?;
                    let result =
                        self.mov_bytes(Reg::get_parameter_index(index), to, parameter.size());
                    Reg::pop();
                    result
                }
                _ => {
                    Reg::set_size(parameter.size());
                    self.mov(
                        Reg::get_parameter_index(index),
                        Reg::STACK { offset: offset },
                    )
                }
            }?;
        }

        visitor.statements.unwrap().accept(self)?;

        // be sure to exit the function even if there is no return at the end!
        self.pop_stack()?;
        self.ret()?;

        Ok(0)
    }
}

impl<'a> Visitor<&ResolvedExpression<'a>, Result<usize, Error>> for Generator {
    fn visit(&mut self, visitor: &ResolvedExpression<'a>) -> Result<usize, Error> {
        match visitor {
            ResolvedExpression::IntLiteral(value) => {
                Reg::set_size(4);
                self.mov(Reg::IMMEDIATE(*value as i64), Reg::current())
            }
            ResolvedExpression::CharLiteral(value) => {
                Reg::set_size(1);
                self.mov(Reg::IMMEDIATE(*value as i64), Reg::current())
            }
            ResolvedExpression::NamedVariable { variable } => match variable.data_type {
                DataType::Struct(_) => self.lea(
                    Reg::STACK {
                        offset: variable.stack_offset,
                    },
                    Reg::current(),
                ),
                x => {
                    Reg::set_size(x.size());
                    self.mov(
                        Reg::STACK {
                            offset: variable.stack_offset,
                        },
                        Reg::current(),
                    )
                }
            },
            ResolvedExpression::Unary {
                expression,
                operation,
                resulting_type,
            } => match operation {
                UnaryOps::NEG => {
                    expression.accept(self)?;
                    let reg = Reg::current();
                    self.emit_sins("neg", reg)
                }
                UnaryOps::COMPLEMENT => {
                    expression.accept(self)?;
                    let reg = Reg::current();
                    self.emit_sins("not", reg)
                }
                UnaryOps::LOGNEG => {
                    expression.accept(self)?;
                    let reg = Reg::current();
                    self.cmp(Reg::IMMEDIATE(0), reg)?;
                    self.mov(Reg::IMMEDIATE(0), reg)?;
                    let prev = Reg::set_size(1);
                    let result = self.emit_sins("sete", reg);
                    Reg::set_size(prev);
                    result
                }
                UnaryOps::REF => match expression {
                    ResolvedExpression::NamedVariable { variable } => self.lea(
                        Reg::STACK {
                            offset: variable.stack_offset,
                        },
                        Reg::current(),
                    ),
                    _ => panic!("should not happen!"),
                },
                UnaryOps::DEREF => match resulting_type {
                    DataType::Struct(_) => expression.accept(self),
                    _ => {
                        expression.accept(self)?;
                        let address = Reg::current().as_address();
                        Reg::set_size(resulting_type.size());
                        self.mov(address, Reg::current())
                    }
                },
                UnaryOps::Cast(_) => expression.accept(self),
            },
            ResolvedExpression::BinaryExpression {
                lhs,
                rhs,
                operation,
                ..
            } => {
                if *operation == BinaryOps::AND || *operation == BinaryOps::OR {
                    return self.generate_and_or(visitor);
                }

                let first_reg = Reg::current();
                lhs.accept(self)?;
                Reg::push();
                rhs.accept(self)?;
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
            ResolvedExpression::FunctionCall(call) => call.accept(self),
            ResolvedExpression::ArrayExpression(arr) => arr.accept(self),
            ResolvedExpression::Indexing {
                index,
                operand,
                data_type,
            } => {
                index.accept(self)?;
                let index = Reg::push();
                operand.accept(self)?;
                let address = Reg::pop();

                Reg::set_size(8);
                self.mul(Reg::IMMEDIATE(visitor.data_type().size() as i64), index)?;
                self.add(index, address)?;

                match data_type {
                    DataType::Struct(_) => {
                        Reg::set_size(data_type.size());
                        self.mov(address, Reg::current())
                    }
                    _ => {
                        let address = address.as_address();
                        Reg::set_size(data_type.size());
                        self.mov(address, Reg::current())
                    }
                }
            }
            ResolvedExpression::Assignment(assignment) => assignment.accept(self),
            // dont have to generate anything
            ResolvedExpression::TypeExpression(_) => Ok(0),
            ResolvedExpression::FieldAccess {
                field_offset,
                data_type,
                operand,
            } => {
                operand.accept(self)?;
                Reg::set_size(data_type.size());
                self.mov(
                    Reg::current().as_address().offset(*field_offset),
                    Reg::current(),
                )
            }
            ResolvedExpression::StructExpresion(expr) => expr.accept(self),
            ResolvedExpression::SizeOf(value) => {
                Reg::set_size(4);
                self.mov(Reg::IMMEDIATE(*value as i64), Reg::current())
            }
            ResolvedExpression::ArrowAccess {
                field_offset,
                data_type,
                operand,
            } => {
                operand.accept(self)?;
                Reg::set_size(data_type.size());
                self.mov(
                    Reg::current().as_address().offset(*field_offset),
                    Reg::current(),
                )
            }
            ResolvedExpression::Cast { expression, .. } => expression.accept(self),
        }
    }
}

impl<'a> Visitor<&ResolvedIfStatement<'a>, Result<usize, Error>> for Generator {
    fn visit(&mut self, visitor: &ResolvedIfStatement<'a>) -> Result<usize, Error> {
        visitor.condition.accept(self)?;

        let (else_part, end) = self.generate_clause_names();
        self.cmp(Reg::IMMEDIATE(0), Reg::current())?;

        self.je(&else_part)?;

        visitor.statements.accept(self)?;

        self.jmp(&end)?;
        self.emit_label(&else_part)?;

        match visitor.else_part {
            ResolvedElsePart::IfStatement(x) => x.accept(self),
            ResolvedElsePart::Compound(x) => x.accept(self),
            ResolvedElsePart::None => Ok(0),
        }?;
        self.emit_label(&end)
    }
}

impl<'a> Visitor<&ResolvedForStatement<'a>, Result<usize, Error>> for Generator {
    fn visit(&mut self, visitor: &ResolvedForStatement<'a>) -> Result<usize, Error> {
        let (body, end, post) = Self::generate_label_names(visitor.label_index);
        visitor.init.accept(self)?;

        self.emit_label(&body)?;

        visitor.condition.accept(self)?;
        self.cmp(Reg::IMMEDIATE(0), Reg::current())?;
        // jump to end of for if the condition is not met anymore
        self.je(&end)?;

        visitor.body.accept(self)?;

        self.emit_label(&post)?;
        if let Some(post) = visitor.post {
            post.accept(self)?;
        }

        self.jmp(&body)?;
        self.emit_label(&end)
    }
}

impl<'a> Visitor<&ResolvedWhileStatement<'a>, Result<usize, Error>> for Generator {
    fn visit(&mut self, visitor: &ResolvedWhileStatement<'a>) -> Result<usize, Error> {
        let (condition, end, _) = Self::generate_label_names(visitor.label_index);
        self.emit_label(&condition)?;
        visitor.condition.accept(self)?;

        self.cmp(Reg::IMMEDIATE(0), Reg::current())?;
        self.je(&end)?;

        visitor.body.accept(self)?;

        self.jmp(&condition)?;
        self.emit_label(&end)
    }
}

impl<'a> Visitor<&ResolvedAssignment<'a>, Result<usize, Error>> for Generator {
    fn visit(&mut self, visitor: &ResolvedAssignment<'a>) -> Result<usize, Error> {
        match visitor {
            ResolvedAssignment::VariableAssignment {
                variable,
                expression,
            } => match variable.data_type {
                DataType::Struct(_) => {
                    expression.accept(self)?;
                    let from = Reg::push();
                    let to = Reg::push();
                    self.lea(
                        Reg::STACK {
                            offset: variable.stack_offset,
                        },
                        to,
                    )?;
                    let result = self.mov_bytes(from, to, variable.data_type.size());
                    Reg::pop();
                    Reg::pop();

                    result
                }
                _ => {
                    expression.accept(self)?;
                    Reg::set_size(variable.data_type.size());
                    self.mov(
                        Reg::current(),
                        Reg::STACK {
                            offset: variable.stack_offset,
                        },
                    )
                }
            },
            ResolvedAssignment::PtrAssignment {
                data_type,
                value,
                address,
            } => match data_type {
                DataType::Struct(_) => {
                    Reg::set_size(data_type.size());
                    address.accept(self)?;
                    let address = Reg::push();
                    value.accept(self)?;
                    let value = Reg::push();

                    let result = self.mov_bytes(value, address, data_type.size());

                    Reg::pop();
                    Reg::pop();
                    result
                }
                _ => {
                    Reg::set_size(data_type.size());
                    address.accept(self)?;
                    let address = Reg::push().as_address();
                    value.accept(self)?;
                    let value = Reg::current();

                    Reg::set_size(data_type.size());
                    let result = self.mov(value, address);
                    Reg::pop();
                    result
                }
            },
            ResolvedAssignment::ArrayAssignment {
                data_type,
                index,
                value,
                address,
            } => match data_type {
                DataType::Struct(_) => {
                    value.accept(self)?;
                    let value = Reg::push();

                    address.accept(self)?;
                    let address = Reg::push();

                    index.accept(self)?;
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
                    value.accept(self)?;
                    let value = Reg::push();

                    address.accept(self)?;
                    let address = Reg::push();

                    index.accept(self)?;
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
            ResolvedAssignment::FieldAssignment {
                field_offset,
                data_type,
                value,
                address,
            } => match data_type {
                DataType::Struct(_) => {
                    address.accept(self)?;
                    let address = Reg::push();

                    value.accept(self)?;
                    let from = Reg::push();

                    // add the offset
                    Reg::set_size(8);
                    self.add(Reg::IMMEDIATE(*field_offset as i64), address)?;

                    Reg::set_size(visitor.data_type().size());
                    let result = self.mov_bytes(from, address, data_type.size());
                    Reg::pop();
                    Reg::pop();
                    result
                }
                _ => {
                    address.accept(self)?;
                    let address = Reg::push();
                    value.accept(self)?;
                    let value = Reg::current();

                    // add the offset
                    Reg::set_size(8);
                    self.add(Reg::IMMEDIATE(*field_offset as i64), address)?;

                    let address = address.as_address();
                    Reg::set_size(visitor.data_type().size());
                    let result = self.mov(value, address);
                    Reg::pop();
                    result
                }
            },
        }
    }
}

impl<'a> Visitor<&ResolvedStructExpression<'a>, Result<usize, Error>> for Generator {
    fn visit(&mut self, visitor: &ResolvedStructExpression<'a>) -> Result<usize, Error> {
        todo!()
        // for assignment in visitor.assignments() {
        //     assignment.accept(self)?;
        // }
        // Reg::set_size(8);
        // self.lea(
        //     Reg::STACK {
        //         offset: *visitor.offset(),
        //     },
        //     Reg::current(),
        // )
    }
}

impl<'a> Visitor<&ResolvedArrayExpression<'a>, Result<usize, Error>> for Generator {
    fn visit(&mut self, visitor: &ResolvedArrayExpression<'a>) -> Result<usize, Error> {
        match visitor {
            ResolvedArrayExpression::StackArray {
                expressions,
                data_type,
            } => {
                todo!()
                // let mut start_offset = *offset;
                // for expr in expressinos {
                //     expr.accept(self)?;
                //     self.mov(
                //         Reg::current(),
                //         Reg::STACK {
                //             offset: start_offset,
                //         },
                //     )?;
                //     start_offset -= base_type.size();
                // }
                // Reg::set_size(8);
                // self.lea(Reg::STACK { offset: *offset }, Reg::current())
            }
            ResolvedArrayExpression::StringLiteral { string, data_type } => {
                // self.emit_string(*label, string)?;
                // Reg::set_size(8);
                // self.emit(&format!("\tlea \t.LC{}(%rip), {}\n", label, Reg::current()))
                todo!()
            }
        }
    }
}

impl<'a> Visitor<&ResolvedFunctionCall<'a>, Result<usize, Error>> for Generator {
    fn visit(&mut self, visitor: &ResolvedFunctionCall<'a>) -> Result<usize, Error> {
        // store parameter in registers
        for (index, parameter) in visitor.parameter.iter().enumerate() {
            parameter.accept(self)?;
            Reg::set_size(parameter.data_type().size());
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
        self.call(visitor.name)?;

        if prev != Reg::R10 {
            Reg::set_size(8);
            while Reg::current() != prev {
                self.emit_sins("pop ", Reg::push())?;
            }
            self.emit_sins("pop ", Reg::current())?;
        }

        Reg::set_size(visitor.return_type.size());
        self.mov(Reg::RAX, Reg::current())
    }
}
