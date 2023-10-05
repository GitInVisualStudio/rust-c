use std::collections::HashMap;

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

use self::ast::{
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
    DataType, Variable,
};
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
    loop_label_index: Vec<i32>,
    string_index: i32,
    stack_offset: usize,
    stack_scope: Vec<usize>,
    frame_size: usize,
    label_index: i32,
}

impl<'a> ScopeBuilder<'a> {
    pub fn new(bump: &'a Bump) -> ScopeBuilder<'a> {
        ScopeBuilder {
            bump: bump,
            scope: Scope::new(),
            current_function: None,
            loop_label_index: Vec::new(),
            stack_offset: 0,
            stack_scope: vec![0],
            label_index: 0,
            string_index: 0,
            frame_size: 0,
        }
    }

    pub fn alloc<T>(&self, t: T) -> &'a mut T {
        self.bump.alloc(t)
    }

    pub fn push(&mut self) {
        self.scope.push();
        self.stack_scope.push(self.stack_offset);
    }

    pub fn pop(&mut self) {
        self.scope.pop();
        self.frame_size = usize::max(self.frame_size, self.stack_offset);
        self.stack_offset = self.stack_scope.pop().unwrap();
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

    pub fn push_variable(&mut self, name: &'a str, type_: DataType<'a>) -> usize {
        self.stack_offset += type_.size();
        let var = Variable::new(self.stack_offset, type_);
        self.scope.push_variable(name, var);
        var.stack_offset
    }

    pub fn get_function(&self, name: &'a str) -> Option<&'a Function<'a>> {
        self.scope.get_function(name)
    }

    pub fn push_function(&mut self, name: &'a str, func: &'a Function<'a>) {
        self.scope.push_function(name, func)
    }

    pub fn next_label_index(&mut self) -> i32 {
        self.label_index += 1;
        self.label_index
    }

    pub fn get_string_label(&mut self) -> i32 {
        self.string_index += 1;
        self.string_index
    }
}

impl<'a> Visitor<&Program<'a>, Result<&'a ResolvedProgram<'a>, Error<'a>>> for ScopeBuilder<'a> {
    fn visit(&mut self, visitor: &Program<'a>) -> Result<&'a ResolvedProgram<'a>, Error<'a>> {
        let mut functions = Vec::new();
        for declaration in &visitor.declarations {
            match declaration {
                Decalrations::Statement(s) => {
                    s.accept(self)?;
                }
                Decalrations::Function(f) => functions.push(f.accept(self)?),
            };
        }
        Ok(self.alloc(ResolvedProgram { functions }))
    }
}

impl<'a> Visitor<&'a Function<'a>, Result<&'a ResolvedFunction<'a>, Error<'a>>>
    for ScopeBuilder<'a>
{
    fn visit(&mut self, visitor: &'a Function<'a>) -> Result<&'a ResolvedFunction<'a>, Error<'a>> {
        self.stack_offset = 0;
        self.frame_size = 0;
        let return_type = visitor.return_type.accept(self)?;
        match self.get_function(visitor.name) {
            Some(x) if x.statements.is_some() == visitor.statements.is_some() => {
                return Err(Error::RedeclarationOfFunction { name: visitor.name })
            }
            Some(x) => {
                let defined_return = x.return_type.accept(self)?;
                if defined_return != return_type {
                    return Err(Error::FunctionDefinitionNotSameAsDeclaration { name: x.name });
                }
                if x.parameter.len() != visitor.parameter.len() {
                    return Err(Error::FunctionDefinitionNotSameAsDeclaration { name: x.name });
                }
                for ((expected, _), (found, _)) in x.parameter.iter().zip(&visitor.parameter) {
                    let expected = expected.accept(self)?;
                    let found = found.accept(self)?;
                    if expected != found {
                        return Err(Error::FunctionDefinitionNotSameAsDeclaration { name: x.name });
                    }
                }
            }
            None => (),
        };
        self.push_function(visitor.name, visitor);
        self.current_function = Some(return_type);
        self.push();

        let mut parameter = Vec::new();
        for (type_, name) in &visitor.parameter {
            let type_ = type_.accept(self)?;
            parameter.push((type_, *name));
            self.push_variable(name, type_);
        }
        let statements;
        match &visitor.statements {
            Some(x) => {
                statements = Some(x.accept(self)?);
                self.pop();
            }
            None => {
                statements = None;
                self.pop();
            }
        };
        Ok(self.alloc(ResolvedFunction {
            name: visitor.name,
            parameter,
            statements,
            frame_size: self.frame_size,
        }))
    }
}

impl<'a> Visitor<&Statement<'a>, Result<&'a ResolvedStatement<'a>, Error<'a>>>
    for ScopeBuilder<'a>
{
    fn visit(&mut self, visitor: &Statement<'a>) -> Result<&'a ResolvedStatement<'a>, Error<'a>> {
        Ok(self.bump.alloc(match visitor {
            Statement::Return(x) => match x {
                Some(expr) => {
                    let expr_type = expr.accept(self)?;
                    match self.current_function {
                        Some(f) => {
                            if f == expr_type.data_type() {
                                ResolvedStatement::Return(Some(expr_type))
                            } else {
                                return Err(Error::ReturnTypeIncorrect {
                                    expected: f,
                                    found: expr_type.data_type(),
                                });
                            }
                        }
                        None => return Err(Error::ReturnWithoutFunction {}),
                    }
                }
                None => match self.current_function {
                    Some(f) => match f {
                        DataType::VOID => ResolvedStatement::Return(None),
                        _ => {
                            return Err(Error::ReturnTypeIncorrect {
                                expected: f,
                                found: DataType::VOID,
                            })
                        }
                    },
                    None => return Err(Error::ReturnWithoutFunction {}),
                },
            },
            Statement::SingleExpression(e) => ResolvedStatement::SingleExpression(e.accept(self)?),
            Statement::Compound(compound) => ResolvedStatement::Compound(compound.accept(self)?),
            Statement::IfStatement(if_statement) => {
                ResolvedStatement::IfStatement(if_statement.accept(self)?)
            }
            Statement::ForStatement(for_statement) => {
                ResolvedStatement::ForStatement(for_statement.accept(self)?)
            }
            Statement::WhileStatement(while_statement) => {
                ResolvedStatement::WhileStatement(while_statement.accept(self)?)
            }
            Statement::TypeDefinition(x) => {
                x.accept(self)?;
                ResolvedStatement::Empty
            }
            Statement::VariableDeclaration {
                name,
                expression,
                assignment,
            } => match self.get_variable(name) {
                Some(_) => return Err(Error::VariableRedefinition { name: name }),
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
                    let offset = self.push_variable(name, type_);
                    match assignment {
                        Some(x) => {
                            let resolved_expr = x.accept(self)?;
                            if resolved_expr.data_type() != type_ {
                                return Err(Error::VariableInitWrong {
                                    expected: type_,
                                    found: resolved_expr.data_type(),
                                    name: name,
                                });
                            }
                            ResolvedStatement::VariableDeclaration {
                                stack_offset: offset,
                                assignment: Some(resolved_expr),
                            }
                        }
                        None => ResolvedStatement::VariableDeclaration {
                            stack_offset: offset,
                            assignment: None,
                        },
                    }
                }
            },
            Statement::Conitnue => {
                let label_index = self.loop_label_index.pop();
                if label_index.is_none() {
                    return Err(Error::ContinueNotInLoop {});
                }
                ResolvedStatement::Continue(label_index.unwrap())
            }
            Statement::Break => {
                let label_index = self.loop_label_index.pop();
                if label_index.is_none() {
                    return Err(Error::BreakNotInLoop {});
                }
                ResolvedStatement::Break(label_index.unwrap())
            }
            Statement::Empty => ResolvedStatement::Empty,
        }))
    }
}

impl<'a> Visitor<&TypeDefinition<'a>, Result<DataType<'a>, Error<'a>>> for ScopeBuilder<'a> {
    fn visit(&mut self, visitor: &TypeDefinition<'a>) -> Result<DataType<'a>, Error<'a>> {
        let resolved = visitor.expression.accept(self)?;
        self.push_type(visitor.name, resolved);
        Ok(resolved)
    }
}

impl<'a> Visitor<&WhileStatement<'a>, Result<&'a ResolvedWhileStatement<'a>, Error<'a>>>
    for ScopeBuilder<'a>
{
    fn visit(
        &mut self,
        visitor: &WhileStatement<'a>,
    ) -> Result<&'a ResolvedWhileStatement<'a>, Error<'a>> {
        let label_index = self.next_label_index();
        self.loop_label_index.push(label_index);

        let condition = visitor.condition.accept(self)?;
        let body = visitor.body.accept(self)?;

        self.loop_label_index.pop();

        Ok(self.alloc(ResolvedWhileStatement {
            condition,
            body,
            label_index,
        }))
    }
}

impl<'a> Visitor<&ForStatement<'a>, Result<&'a ResolvedForStatement<'a>, Error<'a>>>
    for ScopeBuilder<'a>
{
    fn visit(
        &mut self,
        visitor: &ForStatement<'a>,
    ) -> Result<&'a ResolvedForStatement<'a>, Error<'a>> {
        self.push();

        let label_index = self.next_label_index();
        self.loop_label_index.push(label_index);

        let init = visitor.init.accept(self)?;
        let condition = visitor.condition.accept(self)?;
        let post;
        match &visitor.post {
            Some(x) => post = Some(x.accept(self)?),
            None => post = None,
        };
        let body = visitor.body.accept(self)?;

        self.loop_label_index.pop();
        self.pop();
        Ok(self.alloc(ResolvedForStatement {
            init,
            condition,
            post,
            body,
            label_index,
        }))
    }
}

impl<'a> Visitor<&IfStatement<'a>, Result<&'a ResolvedIfStatement<'a>, Error<'a>>>
    for ScopeBuilder<'a>
{
    fn visit(
        &mut self,
        visitor: &IfStatement<'a>,
    ) -> Result<&'a ResolvedIfStatement<'a>, Error<'a>> {
        let condition = visitor.condition.accept(self)?;
        let statements = visitor.statements.accept(self)?;
        let else_part = match &visitor.else_part {
            ElsePart::IfStatement(x) => ResolvedElsePart::IfStatement(x.accept(self)?),
            ElsePart::Compound(x) => ResolvedElsePart::Compound(x.accept(self)?),
            _ => ResolvedElsePart::None,
        };
        Ok(self.alloc(ResolvedIfStatement {
            statements,
            condition,
            else_part,
        }))
    }
}

impl<'a> Visitor<&Compound<'a>, Result<&'a ResolvedCompound<'a>, Error<'a>>> for ScopeBuilder<'a> {
    fn visit(&mut self, visitor: &Compound<'a>) -> Result<&'a ResolvedCompound<'a>, Error<'a>> {
        let mut statements = Vec::new();
        self.push();
        if visitor.statements.len() == 1 {
            match visitor.statements.first().unwrap() {
                Statement::VariableDeclaration { name, .. } => {
                    return Err(Error::SingleStatementMayNotBeDeclaration { name })
                }
                _ => (),
            }
        }
        for s in &visitor.statements {
            statements.push(s.accept(self)?);
        }
        self.pop();
        Ok(self.alloc(ResolvedCompound { statements }))
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
                _ => panic!(
                    "This should not happen! Cannot resolve data-type for token: {:?}",
                    x
                ),
            },
            TypeExpression::Typeof(e) => e.accept(self)?.data_type(),
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

impl<'a> Visitor<&Expression<'a>, Result<&'a ResolvedExpression<'a>, Error<'a>>>
    for ScopeBuilder<'a>
{
    fn visit(&mut self, visitor: &Expression<'a>) -> Result<&'a ResolvedExpression<'a>, Error<'a>> {
        Ok(self.bump.alloc(match visitor {
            Expression::IntLiteral(i) => ResolvedExpression::IntLiteral(*i),
            Expression::CharLiteral(c) => ResolvedExpression::CharLiteral(*c),
            Expression::FunctionCall(function_call) => {
                ResolvedExpression::FunctionCall(function_call.accept(self)?)
            }
            Expression::ArrayExpression(array_expression) => {
                ResolvedExpression::ArrayExpression(array_expression.accept(self)?)
            }
            Expression::StructExpresion(struct_expression) => {
                ResolvedExpression::StructExpresion(struct_expression.accept(self)?)
            }
            Expression::Assignment(assignment) => {
                ResolvedExpression::Assignment(assignment.accept(self)?)
            }
            Expression::TypeExpression(t) => ResolvedExpression::TypeExpression(t.accept(self)?),
            Expression::SizeOf(x) => ResolvedExpression::SizeOf(x.accept(self)?.data_type().size()),
            Expression::FieldAccess { name, operand } => {
                let resolved_operand = operand.accept(self)?;
                match resolved_operand.data_type() {
                    DataType::Struct(x) => match x.field(name) {
                        Some((offset, type_)) => ResolvedExpression::FieldAccess {
                            field_offset: offset,
                            data_type: type_,
                            operand: resolved_operand,
                        },
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
            Expression::ArrowAccess { name, operand } => {
                let resolved_operand = operand.accept(self)?;
                match resolved_operand.data_type() {
                    DataType::PTR(base) => match base {
                        DataType::Struct(x) => match x.field(name) {
                            Some((offset, type_)) => ResolvedExpression::ArrowAccess {
                                field_offset: offset,
                                data_type: type_,
                                operand: resolved_operand,
                            },
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
            Expression::Indexing { operand, index } => {
                let resolved_operand = operand.accept(self)?;
                let resolved_index = index.accept(self)?;
                if !resolved_index.data_type().is_number() {
                    return Err(Error::ArrayIndexNotANumber { index: index });
                }

                match resolved_operand.data_type() {
                    DataType::PTR(x) => ResolvedExpression::Indexing {
                        index: resolved_index,
                        operand: resolved_operand,
                        data_type: *x,
                    },
                    _ => return Err(Error::DerefOfNonPointer { expr: operand }),
                }
            }
            Expression::NamedVariable { name } => match self.get_variable(name) {
                Some(v) => ResolvedExpression::NamedVariable { variable: v },
                None => return Err(Error::UnknownVariable { name }),
            },
            Expression::Unary {
                expression,
                operation,
            } => match operation {
                UnaryOps::REF => {
                    //TODO: have to check if its an l-value
                    // i don't know how yet
                    let expression = expression.accept(self)?;
                    ResolvedExpression::Unary {
                        expression: expression,
                        operation: *operation,
                        resulting_type: DataType::PTR(self.alloc(expression.data_type())),
                    }
                }
                UnaryOps::DEREF => {
                    let resolved_expression = expression.accept(self)?;
                    match resolved_expression.data_type() {
                        DataType::PTR(x) => ResolvedExpression::Unary {
                            expression: resolved_expression,
                            operation: *operation,
                            resulting_type: *x,
                        },
                        _ => return Err(Error::DerefOfNonPointer { expr: expression }),
                    }
                }
                UnaryOps::Cast(expr) => {
                    let type_ = expr.accept(self)?;
                    let expression = expression.accept(self)?;
                    ResolvedExpression::Cast {
                        expression: expression,
                        data_type: type_,
                    }
                }
                _ => {
                    let resolved_expression = expression.accept(self)?;
                    if !resolved_expression.data_type().is_number() {
                        return Err(Error::UnaryOperandNotNumber {
                            expression,
                            operation: *operation,
                        });
                    }
                    ResolvedExpression::Unary {
                        expression: resolved_expression,
                        operation: *operation,
                        resulting_type: resolved_expression.data_type(),
                    }
                }
            },
            Expression::BinaryExpression {
                lhs,
                rhs,
                operation,
            } => {
                let resolved_lhs = lhs.accept(self)?;
                let resolved_rhs = rhs.accept(self)?;
                let lhs_data = resolved_lhs.data_type();
                let rhs_data = resolved_rhs.data_type();
                if lhs_data != rhs_data && !lhs_data.can_convert(rhs_data) {
                    return Err(Error::OperandsDifferentDatatypes { lhs: lhs, rhs: rhs });
                }
                ResolvedExpression::BinaryExpression {
                    lhs: resolved_lhs,
                    rhs: resolved_rhs,
                    operation: *operation,
                    resulting_type: resolved_lhs.data_type(),
                }
            }
        }))
    }
}

impl<'a> Visitor<&'a Assignment<'a>, Result<&'a ResolvedAssignment<'a>, Error<'a>>>
    for ScopeBuilder<'a>
{
    fn visit(
        &mut self,
        visitor: &'a Assignment<'a>,
    ) -> Result<&'a ResolvedAssignment<'a>, Error<'a>> {
        Ok(self.bump.alloc(match visitor {
            Assignment::VariableAssignment { name, expression } => match self.get_variable(name) {
                Some(x) => {
                    let expr = expression.accept(self)?;
                    if expr.data_type() != x.data_type && !expr.data_type().can_convert(x.data_type)
                    {
                        return Err(Error::CannotAssignVariable {
                            assignment: expression,
                            name: name,
                            variable: x,
                        });
                    }
                    ResolvedAssignment::StackAssignment {
                        variable: x,
                        expression: expr,
                    }
                }
                None => return Err(Error::UnknownVariable { name: name }),
            },
            Assignment::PtrAssignment { value, address } => {
                let resolved_address = address.accept(self)?;
                let resolved_value = value.accept(self)?;
                match resolved_address.data_type() {
                    DataType::PTR(base) => {
                        if *base != resolved_value.data_type()
                            && !resolved_value.data_type().can_convert(*base)
                        {
                            return Err(Error::CannotAssign {
                                from: value,
                                to: address,
                            });
                        }
                        ResolvedAssignment::PtrAssignment {
                            value: resolved_value,
                            address: resolved_value,
                            data_type: *base,
                        }
                    }
                    _ => return Err(Error::DerefOfNonPointer { expr: address }),
                }
            }
            Assignment::ArrayAssignment {
                index,
                value,
                address,
            } => {
                let resolved_index = index.accept(self)?;
                if !resolved_index.data_type().is_number() {
                    return Err(Error::ArrayIndexNotANumber { index: index });
                }
                let resolved_value = value.accept(self)?;
                let resolved_address = address.accept(self)?;
                match resolved_address.data_type() {
                    DataType::PTR(base) => {
                        if *base != resolved_value.data_type()
                            && !resolved_value.data_type().can_convert(*base)
                        {
                            return Err(Error::CannotAssign {
                                from: value,
                                to: address,
                            });
                        }
                        ResolvedAssignment::ArrayAssignment {
                            index: resolved_index,
                            value: resolved_value,
                            address: resolved_address,
                            data_type: *base,
                        }
                    }
                    _ => return Err(Error::DerefOfNonPointer { expr: address }),
                }
            }
            Assignment::FieldAssignment {
                name,
                value,
                address,
            } => {
                let resolved_address = address.accept(self)?;
                let resolved_value = value.accept(self)?;
                match resolved_address.data_type() {
                    DataType::Struct(x) => match x.field(name) {
                        Some((field_offset, field_type)) => {
                            if field_type != resolved_value.data_type()
                                && !resolved_value.data_type().can_convert(field_type)
                            {
                                return Err(Error::CannotAssign {
                                    from: value,
                                    to: address,
                                });
                            }
                            ResolvedAssignment::FieldAssignment {
                                field_offset: field_offset,
                                value: resolved_value,
                                address: resolved_address,
                                data_type: field_type,
                            }
                        }
                        None => {
                            return Err(Error::UnknownField {
                                expression: address,
                                name: name,
                            })
                        }
                    },
                    DataType::PTR(x) => match x {
                        DataType::Struct(x) => match x.field(name) {
                            Some((field_offset, field_type)) => {
                                if field_type != resolved_value.data_type()
                                    && !resolved_value.data_type().can_convert(field_type)
                                {
                                    return Err(Error::CannotAssign {
                                        from: value,
                                        to: address,
                                    });
                                }
                                ResolvedAssignment::FieldAssignment {
                                    field_offset: field_offset,
                                    value: resolved_value,
                                    address: resolved_address,
                                    data_type: field_type,
                                }
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
                    },
                    _ => {
                        return Err(Error::AccessNonStruct {
                            expression: &address,
                        })
                    }
                }
            }
        }))
    }
}

impl<'a> Visitor<&StructExpression<'a>, Result<&'a ResolvedStructExpression<'a>, Error<'a>>>
    for ScopeBuilder<'a>
{
    fn visit(
        &mut self,
        visitor: &StructExpression<'a>,
    ) -> Result<&'a ResolvedStructExpression<'a>, Error<'a>> {
        let mut fields = Vec::new();
        let mut named_fields = Vec::new();
        for (name, type_) in visitor.fields.iter().rev() {
            let type_ = type_.accept(self)?;
            let data_type = type_.data_type();
            self.stack_offset += data_type.size();
            named_fields.push((*name, data_type));
            let assignment = ResolvedAssignment::StackAssignment {
                variable: Variable {
                    stack_offset: self.stack_offset,
                    data_type,
                },
                expression: type_,
            };
            fields.push(&*self.alloc(assignment))
        }
        named_fields.reverse();
        let data_type = Struct::new(named_fields);
        let data_type = DataType::Struct(self.alloc(data_type));

        Ok(self.alloc(ResolvedStructExpression {
            fields,
            data_type,
            stack_offset: self.stack_offset,
        }))
    }
}

impl<'a> Visitor<&ArrayExpression<'a>, Result<&'a ResolvedArrayExpression<'a>, Error<'a>>>
    for ScopeBuilder<'a>
{
    fn visit(
        &mut self,
        visitor: &ArrayExpression<'a>,
    ) -> Result<&'a ResolvedArrayExpression<'a>, Error<'a>> {
        Ok(self.bump.alloc(match visitor {
            ArrayExpression::StackArray { expressions } => {
                let first_expr = match expressions.first() {
                    Some(x) => x.accept(self)?,
                    None => return Err(Error::EmptyArray {}),
                };
                let mut resolved_expressions = Vec::new();
                for expr in expressions.iter().rev() {
                    let expr = expr.accept(self)?;
                    self.stack_offset += expr.data_type().size();
                    if expr.data_type() != first_expr.data_type() {
                        return Err(Error::ArrayOfDifferentTypes {});
                    }
                    let assignment = ResolvedAssignment::StackAssignment {
                        variable: Variable {
                            stack_offset: self.stack_offset,
                            data_type: expr.data_type(),
                        },
                        expression: expr,
                    };
                    resolved_expressions.push(&*self.alloc(assignment));
                }
                ResolvedArrayExpression::StackArray {
                    expressions: resolved_expressions,
                    data_type: DataType::PTR(self.alloc(first_expr.data_type())),
                    stack_offset: self.stack_offset,
                }
            }
            ArrayExpression::StringLiteral { string } => ResolvedArrayExpression::StringLiteral {
                string,
                data_type: DataType::PTR(self.alloc(DataType::CHAR)),
                string_label_index: self.get_string_label(),
            },
        }))
    }
}

impl<'a> Visitor<&FunctionCall<'a>, Result<&'a ResolvedFunctionCall<'a>, Error<'a>>>
    for ScopeBuilder<'a>
{
    fn visit(
        &mut self,
        visitor: &FunctionCall<'a>,
    ) -> Result<&'a ResolvedFunctionCall<'a>, Error<'a>> {
        match self.get_function(visitor.name) {
            Some(func) => {
                if func.parameter.len() != visitor.parameter.len() {
                    return Err(Error::ParameterCountMismatch {
                        function: visitor.name,
                        expected: func.parameter.len(),
                        found: visitor.parameter.len(),
                    });
                }
                let mut resolved_parameter = Vec::new();
                for (found, (expected, parameter_name)) in
                    visitor.parameter.iter().zip(&func.parameter)
                {
                    let expected = expected.accept(self)?;
                    let found = found.accept(self)?;
                    if expected != found.data_type() && !found.data_type().can_convert(expected) {
                        return Err(Error::ParameterTypeMismatch {
                            function: visitor.name,
                            expected: expected,
                            found: found.data_type(),
                            parameter_name,
                        });
                    }
                    resolved_parameter.push(found);
                }
                let return_type = func.return_type.accept(self)?;

                Ok(self.alloc(ResolvedFunctionCall {
                    name: visitor.name,
                    parameter: resolved_parameter,
                    return_type,
                }))
            }
            None => Err(Error::UnkownFunction { name: visitor.name }),
        }
    }
}
