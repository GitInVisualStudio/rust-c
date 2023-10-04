use std::{collections::HashMap, sync::Arc};

use bumpalo::Bump;

use crate::{
    error::Error,
    lexer::tokens::TokenKind,
    parser::ast::{
        ArrayExpression, Assignment, Compound, Decalrations, ElsePart, Expression, ForStatement,
        Function, FunctionCall, IfStatement, Program, Statement, StructExpression, TypeDefinition,
        TypeExpression, UnaryOps, WhileStatement,
    },
    scope_builder::ast::data_type::Struct,
    visitor::{Visitable, Visitor},
};

use self::ast::{DataType, Variable};
pub mod ast;

pub struct Scope<'a> {
    types: Vec<HashMap<&'a str, DataType<'a>>>,
    variables: Vec<HashMap<&'a str, Variable<'a>>>,
    functions: Vec<HashMap<&'a str, &'a Function<'a>>>,
}

impl<'a> Scope<'a> {
    pub fn new() -> Scope<'a> {
        Scope {
            types: vec![HashMap::new()],
            variables: vec![HashMap::new()],
            functions: vec![HashMap::new()],
        }
    }

    pub fn push(&mut self) {
        self.types.push(HashMap::new());
        self.variables.push(HashMap::new());
        self.functions.push(HashMap::new());
    }

    pub fn pop(&mut self) {
        self.types.pop();
        self.variables.pop();
        self.functions.push(HashMap::new());
    }

    pub fn get_type(&self, name: &'a str) -> Option<DataType<'a>> {
        for map in self.types.iter().rev() {
            if let Some(data_type) = map.get(name) {
                return Some(*data_type);
            }
        }
        None
    }

    pub fn push_type(&mut self, name: &'a str, data_type: DataType<'a>) {
        let map = self.types.last_mut().unwrap();
        map.insert(name, data_type);
    }

    pub fn get_variable(&self, name: &'a str) -> Option<Variable<'a>> {
        for map in self.variables.iter().rev() {
            if let Some(data_type) = map.get(name) {
                return Some(*data_type);
            }
        }
        None
    }

    pub fn push_variable(&mut self, name: &'a str, var: Variable<'a>) {
        let map = self.variables.last_mut().unwrap();
        map.insert(name, var);
    }

    pub fn get_function(&self, name: &'a str) -> Option<&'a Function<'a>> {
        for map in self.functions.iter().rev() {
            if let Some(data_type) = map.get(name) {
                return Some(*data_type);
            }
        }
        None
    }

    pub fn push_function(&mut self, name: &'a str, func: &'a Function<'a>) {
        let map = self.functions.last_mut().unwrap();
        map.insert(name, func);
    }
}

pub struct ScopeBuilder<'a> {
    bump: &'a Bump,
    scope: Scope<'a>,
    current_function: Option<DataType<'a>>,
    stack_offset: usize,
}

impl<'a> ScopeBuilder<'a> {
    pub fn new(bump: &'a Bump) -> ScopeBuilder<'a> {
        ScopeBuilder {
            bump: bump,
            scope: Scope::new(),
            current_function: None,
            stack_offset: 0,
        }
    }

    pub fn alloc<T>(&self, t: T) -> &'a mut T {
        self.bump.alloc(t)
    }

    pub fn push(&mut self) {
        self.scope.push()
    }

    pub fn pop(&mut self) {
        self.scope.pop()
    }

    pub fn get_type(&self, name: &'a str) -> Option<DataType<'a>> {
        self.scope.get_type(name)
    }

    pub fn push_type(&mut self, name: &'a str, data_type: DataType<'a>) {
        self.scope.push_type(name, data_type)
    }

    pub fn get_variable(&self, name: &'a str) -> Option<Variable<'a>> {
        self.scope.get_variable(name)
    }

    pub fn push_variable(&mut self, name: &'a str, type_: DataType<'a>) {
        let var = Variable::new(self.stack_offset, type_);
        self.stack_offset += type_.size();
        self.scope.push_variable(name, var)
    }

    pub fn get_function(&self, name: &'a str) -> Option<&'a Function<'a>> {
        self.scope.get_function(name)
    }

    pub fn push_function(&mut self, name: &'a str, func: &'a Function<'a>) {
        self.scope.push_function(name, func)
    }
}
impl<'a> Visitor<&Program<'a>, Result<DataType<'a>, Error<'a>>> for ScopeBuilder<'a> {
    fn visit(&mut self, visitor: &Program<'a>) -> Result<DataType<'a>, Error<'a>> {
        for declaration in &visitor.declarations {
            match declaration {
                Decalrations::Statement(s) => s.accept(self)?,
                Decalrations::Function(f) => f.accept(self)?,
            };
        }
        Ok(DataType::VOID)
    }
}

impl<'a> Visitor<&'a Function<'a>, Result<DataType<'a>, Error<'a>>> for ScopeBuilder<'a> {
    fn visit(&mut self, visitor: &'a Function<'a>) -> Result<DataType<'a>, Error<'a>> {
        self.push_function(visitor.name, visitor);

        let return_type = visitor.return_type.accept(self)?;
        self.current_function = Some(return_type);
        self.push();
        for (type_, name) in &visitor.parameter {
            let type_ = type_.accept(self)?;
            self.push_variable(name, type_)
        }
        match &visitor.statements {
            Some(x) => {
                x.accept(self)?;
                self.pop();
                Ok(return_type)
            }
            None => {
                self.pop();
                Ok(return_type)
            }
        }
    }
}

impl<'a> Visitor<&Statement<'a>, Result<DataType<'a>, Error<'a>>> for ScopeBuilder<'a> {
    fn visit(&mut self, visitor: &Statement<'a>) -> Result<DataType<'a>, Error<'a>> {
        match visitor {
            Statement::Return(x) => match x {
                Some(expr) => {
                    let expr_type = expr.accept(self)?;
                    match self.current_function {
                        Some(f) => {
                            if f == expr_type {
                                Ok(expr_type)
                            } else {
                                Err(Error::ReturnTypeIncorrect {
                                    expected: f,
                                    found: expr_type,
                                })
                            }
                        }
                        None => Err(Error::ReturnWithoutFunction {}),
                    }
                }
                None => match self.current_function {
                    Some(f) => match f {
                        DataType::VOID => Ok(f),
                        _ => Err(Error::ReturnTypeIncorrect {
                            expected: f,
                            found: DataType::VOID,
                        }),
                    },
                    None => Err(Error::ReturnWithoutFunction {}),
                },
            },
            Statement::SingleExpression(e) => e.accept(self),
            Statement::Compound(compound) => compound.accept(self),
            Statement::IfStatement(if_statement) => if_statement.accept(self),
            Statement::ForStatement(for_statement) => for_statement.accept(self),
            Statement::WhileStatement(while_statement) => while_statement.accept(self),
            Statement::TypeDefinition(type_def) => type_def.accept(self),
            Statement::VariableDeclaration {
                name,
                expression,
                assignment,
            } => match self.get_variable(name) {
                Some(_) => Err(Error::VariableRedefinition { name: name }),
                None => {
                    if self.current_function.is_none() {
                        return Err(Error::VariableDeclarationOutsideOfFunction { name: name });
                    }
                    let type_ = expression.accept(self)?;
                    if type_.size() == 0 {
                        return Err(Error::VariableOfUnkownSize {
                            name: name,
                            data_type: type_,
                        });
                    }
                    self.push_variable(name, type_);
                    match assignment {
                        Some(x) if x.accept(self)? != type_ => Err(Error::VariableInitWrong {
                            expected: type_,
                            found: x.accept(self)?,
                            name: name,
                        }),
                        _ => Ok(type_),
                    }
                }
            },
            _ => Ok(DataType::VOID),
        }
    }
}

impl<'a> Visitor<&TypeDefinition<'a>, Result<DataType<'a>, Error<'a>>> for ScopeBuilder<'a> {
    fn visit(&mut self, visitor: &TypeDefinition<'a>) -> Result<DataType<'a>, Error<'a>> {
        let resolved = visitor.expression.accept(self)?;
        self.push_type(visitor.name, resolved);
        Ok(resolved)
    }
}

impl<'a> Visitor<&WhileStatement<'a>, Result<DataType<'a>, Error<'a>>> for ScopeBuilder<'a> {
    fn visit(&mut self, visitor: &WhileStatement<'a>) -> Result<DataType<'a>, Error<'a>> {
        visitor.condition.accept(self)?;
        visitor.body.accept(self)
    }
}

impl<'a> Visitor<&ForStatement<'a>, Result<DataType<'a>, Error<'a>>> for ScopeBuilder<'a> {
    fn visit(&mut self, visitor: &ForStatement<'a>) -> Result<DataType<'a>, Error<'a>> {
        self.push();
        visitor.init.accept(self)?;
        visitor.condition.accept(self)?;
        match &visitor.post {
            Some(x) => x.accept(self)?,
            None => DataType::VOID,
        };
        let result = visitor.body.accept(self);
        self.pop();
        result
    }
}

impl<'a> Visitor<&IfStatement<'a>, Result<DataType<'a>, Error<'a>>> for ScopeBuilder<'a> {
    fn visit(&mut self, visitor: &IfStatement<'a>) -> Result<DataType<'a>, Error<'a>> {
        visitor.condition.accept(self)?;
        visitor.statements.accept(self)?;
        match &visitor.else_part {
            ElsePart::IfStatement(x) => x.accept(self)?,
            ElsePart::Compound(x) => x.accept(self)?,
            _ => DataType::VOID,
        };
        Ok(DataType::VOID)
    }
}

impl<'a> Visitor<&Compound<'a>, Result<DataType<'a>, Error<'a>>> for ScopeBuilder<'a> {
    fn visit(&mut self, visitor: &Compound<'a>) -> Result<DataType<'a>, Error<'a>> {
        self.push();
        for s in &visitor.statements {
            s.accept(self)?;
        }
        self.pop();
        Ok(DataType::VOID)
    }
}

impl<'a> Visitor<&TypeExpression<'a>, Result<DataType<'a>, Error<'a>>> for ScopeBuilder<'a> {
    fn visit(&mut self, visitor: &TypeExpression<'a>) -> Result<DataType<'a>, Error<'a>> {
        let data_type = match visitor {
            TypeExpression::Primitive(x) => match x {
                TokenKind::INT => DataType::INT,
                TokenKind::CHAR => DataType::CHAR,
                TokenKind::LONG => DataType::LONG,
                TokenKind::VOID => DataType::VOID,
                _ => panic!("TODO:!"),
            },
            TypeExpression::Typeof(e) => e.accept(self)?,
            TypeExpression::Named(name) => match self.get_type(name) {
                Some(x) => x,
                None => return Err(Error::UnknownType { type_name: name }),
            },
            TypeExpression::Struct { name, fields } => match self.get_type(name) {
                Some(x) if x == DataType::EmptyStruct => {
                    let mut resolved_fields = Vec::new();

                    for (name, type_expr) in fields {
                        resolved_fields.push((*name, type_expr.accept(self)?))
                    }
                    let struct_ = Struct::new(resolved_fields);
                    let struct_ = self.alloc(struct_);
                    DataType::Struct(struct_)
                }
                Some(_) => return Err(Error::StructRedefinition { name: name }),
                None => {
                    self.push_type(name, DataType::EmptyStruct);
                    let mut resolved_fields = Vec::new();

                    for (field_name, type_expr) in fields {
                        if fields.iter().filter(|x| x.0 == *field_name).count() != 1 {
                            return Err(Error::StructFieldRedefinition {
                                struct_name: name,
                                field_name,
                            });
                        }
                        let type_ = type_expr.accept(self)?;
                        if type_.size() == 0 {
                            return Err(Error::StructFieldUnkownSize {
                                struct_name: name,
                                field_name,
                            });
                        }
                        resolved_fields.push((*field_name, type_))
                    }
                    let struct_ = Struct::new(resolved_fields);
                    let struct_ = self.alloc(struct_);
                    let type_ = DataType::Struct(struct_);
                    self.push_type(name, type_);
                    DataType::Struct(struct_)
                }
            },
            TypeExpression::NamedStruct(name) => match self.get_type(name) {
                Some(x) => x,
                None => {
                    self.push_type(name, DataType::EmptyStruct);
                    DataType::EmptyStruct
                }
            },
            TypeExpression::Pointer(expr) => {
                let resolved = expr.accept(self)?;
                let base = self.alloc(resolved);
                DataType::PTR(base)
            }
        };
        Ok(data_type)
    }
}

impl<'a> Visitor<&Expression<'a>, Result<DataType<'a>, Error<'a>>> for ScopeBuilder<'a> {
    fn visit(&mut self, visitor: &Expression<'a>) -> Result<DataType<'a>, Error<'a>> {
        Ok(match visitor {
            Expression::IntLiteral(_) => DataType::INT,
            Expression::CharLiteral(_) => DataType::CHAR,
            Expression::FunctionCall(function_call) => function_call.accept(self)?,
            Expression::ArrayExpression(array_expression) => array_expression.accept(self)?,
            Expression::StructExpresion(struct_expression) => struct_expression.accept(self)?,
            Expression::Assignment(assignment) => assignment.accept(self)?,
            Expression::TypeExpression(t) => t.accept(self)?,
            Expression::SizeOf(_) => DataType::INT,
            Expression::FieldAccess { name, operand } => {
                let base = operand.accept(self)?;
                match base {
                    DataType::PTR(type_) => match type_ {
                        DataType::Struct(x) => match x.field(name) {
                            Some(x) => x,
                            None => {
                                return Err(Error::UnknownField {
                                    expression: operand,
                                    name: name,
                                })
                            }
                        },
                        _ => {
                            return Err(Error::AccessNonStruct {
                                expression: operand,
                            })
                        }
                    },
                    _ => return Err(Error::DerefOfNonPointer { expr: operand }),
                }
            }
            Expression::ArrowAccess { name, operand } => {
                let type_ = operand.accept(self)?;
                match type_ {
                    DataType::Struct(x) => match x.field(name) {
                        Some(x) => x,
                        None => {
                            return Err(Error::UnknownField {
                                expression: operand,
                                name: name,
                            })
                        }
                    },
                    _ => {
                        return Err(Error::AccessNonStruct {
                            expression: operand,
                        })
                    }
                }
            }
            Expression::Indexing { operand, .. } => {
                let base = operand.accept(self)?;
                match base {
                    DataType::PTR(x) => *x,
                    _ => return Err(Error::DerefOfNonPointer { expr: operand }),
                }
            }
            Expression::NamedVariable { name } => match self.get_variable(name) {
                Some(v) => v.data_type(),
                None => return Err(Error::UnknownVariable { name }),
            },
            Expression::Unary {
                expression,
                operation,
            } => match operation {
                UnaryOps::REF => {
                    let base = expression.accept(self)?;
                    DataType::PTR(self.alloc(base))
                }
                UnaryOps::DEREF => {
                    let base = expression.accept(self)?;
                    match base {
                        DataType::PTR(x) => *x,
                        _ => return Err(Error::DerefOfNonPointer { expr: expression }),
                    }
                }
                UnaryOps::Cast(_) => todo!(),
                _ => expression.accept(self)?,
            },
            Expression::BinaryExpression { lhs, rhs, .. } => {
                let lhs_type = lhs.accept(self)?;
                let rhs_type = rhs.accept(self)?;
                if lhs_type != rhs_type {
                    return Err(Error::OperandsDifferentDatatypes { lhs: lhs, rhs: rhs });
                }
                lhs_type
            }
        })
    }
}

impl<'a> Visitor<&'a Assignment<'a>, Result<DataType<'a>, Error<'a>>> for ScopeBuilder<'a> {
    fn visit(&mut self, visitor: &'a Assignment<'a>) -> Result<DataType<'a>, Error<'a>> {
        Ok(match visitor {
            Assignment::VariableAssignment { name, expression } => match self.get_variable(name) {
                Some(x) => {
                    let expr_type = expression.accept(self)?;
                    if expr_type != x.data_type() {
                        return Err(Error::CannotAssignVariable {
                            assignment: expression,
                            variable: name,
                        });
                    }
                    expr_type
                }
                None => return Err(Error::UnknownVariable { name: name }),
            },
            Assignment::PtrAssignment { value, address } => {
                let base = address.accept(self)?;
                let value_type = value.accept(self)?;
                match base {
                    DataType::PTR(base) => {
                        if *base != value_type {
                            return Err(Error::CannotAssign {
                                from: value,
                                to: address,
                            });
                        }
                        *base
                    }
                    _ => return Err(Error::DerefOfNonPointer { expr: address }),
                }
            }
            Assignment::ArrayAssignment {
                index,
                value,
                address,
            } => {
                let index_type = index.accept(self)?;
                if !index_type.is_number() {
                    return Err(Error::ArrayIndexNotANumber { index: index });
                }
                let value_type = value.accept(self)?;
                let base = address.accept(self)?;
                match base {
                    DataType::PTR(base) => {
                        if *base != value_type {
                            return Err(Error::CannotAssign {
                                from: value,
                                to: address,
                            });
                        }
                        *base
                    }
                    _ => return Err(Error::DerefOfNonPointer { expr: address }),
                }
            }
            Assignment::FieldAssignment {
                name,
                value,
                address,
            } => {
                let struct_type = address.accept(self)?;
                let value_type = value.accept(self)?;
                match struct_type {
                    DataType::Struct(x) => match x.field(name) {
                        Some(field_type) => {
                            if field_type != value_type {
                                return Err(Error::CannotAssign {
                                    from: value,
                                    to: address,
                                });
                            }
                            field_type
                        }
                        None => {
                            return Err(Error::UnknownField {
                                expression: address,
                                name: name,
                            })
                        }
                    },
                    _ => {
                        return Err(Error::AccessNonStruct {
                            expression: &address,
                        })
                    }
                }
            }
        })
    }
}

impl<'a> Visitor<&StructExpression<'a>, Result<DataType<'a>, Error<'a>>> for ScopeBuilder<'a> {
    fn visit(&mut self, visitor: &StructExpression<'a>) -> Result<DataType<'a>, Error<'a>> {
        let mut resolved_fields = Vec::new();
        for (name, type_) in &visitor.fields {
            resolved_fields.push((*name, type_.accept(self)?));
        }
        let struct_ = Struct::new(resolved_fields);
        Ok(DataType::Struct(self.alloc(struct_)))
    }
}

impl<'a> Visitor<&ArrayExpression<'a>, Result<DataType<'a>, Error<'a>>> for ScopeBuilder<'a> {
    fn visit(&mut self, visitor: &ArrayExpression<'a>) -> Result<DataType<'a>, Error<'a>> {
        Ok(match visitor {
            ArrayExpression::StackArray { expressions } => {
                let base_type = match expressions.first() {
                    Some(x) => x.accept(self)?,
                    None => return Err(Error::EmptyArray {}),
                };
                for expr in expressions.iter().skip(1) {
                    if expr.accept(self)? != base_type {
                        return Err(Error::ArrayOfDifferentTypes {});
                    }
                }
                DataType::PTR(self.alloc(base_type))
            }
            ArrayExpression::StringLiteral { .. } => DataType::PTR(self.alloc(DataType::CHAR)),
        })
    }
}

impl<'a> Visitor<&FunctionCall<'a>, Result<DataType<'a>, Error<'a>>> for ScopeBuilder<'a> {
    fn visit(&mut self, visitor: &FunctionCall<'a>) -> Result<DataType<'a>, Error<'a>> {
        match self.get_function(visitor.name) {
            Some(func) => {
                if func.parameter.len() != visitor.parameter.len() {
                    return Err(Error::ParameterCountMismatch {
                        function: visitor.name,
                        expected: func.parameter.len(),
                        found: visitor.parameter.len(),
                    });
                }
                for (found, (expected, parameter_name)) in
                    visitor.parameter.iter().zip(&func.parameter)
                {
                    let expected = expected.accept(self)?;
                    let found = found.accept(self)?;
                    if expected != found {
                        return Err(Error::ParamterTypeMismatch {
                            function: visitor.name,
                            expected: expected,
                            found: found,
                            parameter_name,
                        });
                    }
                }
                Ok(func.return_type.accept(self)?)
            }
            None => Err(Error::UnkownFunction { name: visitor.name }),
        }
    }
}
