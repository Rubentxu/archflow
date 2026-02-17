// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Expression DSL
//
// This module provides a lightweight bytecode VM for evaluating complex logical
// expressions in Logic Bricks controllers.
//
// Expression DSL allows complex boolean expressions and arithmetic comparisons
// without writing JavaScript at runtime. Expressions are compiled to bytecode
// and evaluated efficiently using a stack-based virtual machine.
//
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
use alloc::vec;

use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Operation codes for the expression bytecode VM.
///
/// Each opcode represents a single operation that the VM can execute.
/// The VM uses a stack-based architecture where operands are pushed and popped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum OpCode {
    // ═══════════════════════════════════════════════════════════════════════════
    // STACK OPERATIONS
    // ═══════════════════════════════════════════════════════════════════════════
    /// No-op: does nothing
    Nop = 0,

    /// Push a constant value onto the stack
    /// Operand: constant index (u8)
    PushConst = 1,

    /// Push a variable value onto the stack
    /// Operand: variable index (u8)
    LoadVar = 2,

    /// Store top of stack into a variable
    /// Operand: variable index (u8)
    StoreVar = 3,

    // ═══════════════════════════════════════════════════════════════════════════
    // ARITHMETIC OPERATIONS
    // ═══════════════════════════════════════════════════════════════════════════
    /// Addition: pop b, pop a, push a + b
    Add = 10,

    /// Subtraction: pop b, pop a, push a - b
    Subtract = 11,

    /// Multiplication: pop b, pop a, push a * b
    Multiply = 12,

    /// Division: pop b, pop a, push a / b
    Divide = 13,

    /// Modulo: pop b, pop a, push a % b
    Modulo = 14,

    /// Negation: pop a, push -a
    Negate = 15,

    // ═══════════════════════════════════════════════════════════════════════════
    // COMPARISON OPERATIONS
    // ═══════════════════════════════════════════════════════════════════════════
    /// Equality: pop b, pop a, push a == b
    Equal = 20,

    /// Inequality: pop b, pop a, push a != b
    NotEqual = 21,

    /// Greater than: pop b, pop a, push a > b
    Greater = 22,

    /// Greater than or equal: pop b, pop a, push a >= b
    GreaterEqual = 23,

    /// Less than: pop b, pop a, push a < b
    Less = 24,

    /// Less than or equal: pop b, pop a, push a <= b
    LessEqual = 25,

    // ═══════════════════════════════════════════════════════════════════════════
    // LOGICAL OPERATIONS
    // ═══════════════════════════════════════════════════════════════════════════
    /// Logical AND: pop b, pop a, push a && b
    And = 30,

    /// Logical OR: pop b, pop a, push a || b
    Or = 31,

    /// Logical NOT: pop a, push !a
    Not = 32,

    // ═══════════════════════════════════════════════════════════════════════════
    // CONTROL FLOW
    // ═══════════════════════════════════════════════════════════════════════════
    /// Unconditional jump
    /// Operand: target address (u16)
    Jump = 40,

    /// Conditional jump: pop condition, jump if false
    /// Operand: target address (u16)
    JumpIfFalse = 41,

    /// Conditional jump: pop condition, jump if true
    /// Operand: target address (u16)
    JumpIfTrue = 42,

    // ═══════════════════════════════════════════════════════════════════════════
    // SPECIAL OPERATIONS
    // ═══════════════════════════════════════════════════════════════════════════
    /// Get entity property
    /// Operand: property name index (u8)
    GetProperty = 50,

    /// Get sensor state
    /// Operand: sensor index (u8)
    GetSensor = 51,

    /// Get timestamp from context
    GetTimestamp = 52,

    /// Get entity ID from context
    GetEntityId = 53,
}

impl OpCode {
    /// Returns the number of operands this opcode expects
    #[inline(always)]
    pub fn operand_count(self) -> usize {
        match self {
            Self::Nop => 0,
            Self::PushConst => 1,
            Self::LoadVar => 1,
            Self::StoreVar => 1,
            Self::Add | Self::Subtract | Self::Multiply | Self::Divide | Self::Modulo => 0,
            Self::Negate | Self::Not => 0,
            Self::Equal
            | Self::NotEqual
            | Self::Greater
            | Self::GreaterEqual
            | Self::Less
            | Self::LessEqual => 0,
            Self::And | Self::Or => 0,
            Self::Jump | Self::JumpIfFalse | Self::JumpIfTrue => 2,
            Self::GetProperty => 1,
            Self::GetSensor => 1,
            Self::GetTimestamp => 0,
            Self::GetEntityId => 0,
        }
    }
}

/// Compiled expression containing bytecode and metadata.
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct CompiledExpression {
    pub bytecode: Vec<u8>,
    pub constants: Vec<f64>,
    pub variables: Vec<String>,
    pub max_stack: usize,
}

impl CompiledExpression {
    #[inline(always)]
    pub fn new(
        bytecode: Vec<u8>,
        constants: Vec<f64>,
        variables: Vec<String>,
        max_stack: usize,
    ) -> Self {
        Self {
            bytecode,
            constants,
            variables,
            max_stack,
        }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.bytecode.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.bytecode.is_empty()
    }
}

/// Expression controller for complex logic evaluation.
#[derive(Clone, Debug)]
pub struct ExpressionController {
    pub source: String,
    pub compiled: Option<CompiledExpression>,
}

impl ExpressionController {
    #[inline(always)]
    pub fn new(source: String) -> Self {
        Self {
            source,
            compiled: None,
        }
    }

    pub fn get_compiled(&mut self) -> Option<&CompiledExpression> {
        self.compiled.as_ref()
    }

    pub fn set_compiled(&mut self, compiled: CompiledExpression) {
        self.compiled = Some(compiled);
    }
}

/// Value type for the expression VM
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    Number(f64),
}

impl Value {
    #[inline(always)]
    pub fn as_bool(self) -> bool {
        match self {
            Self::Bool(b) => b,
            Self::Number(n) => n != 0.0,
        }
    }

    #[inline(always)]
    pub fn as_number(self) -> f64 {
        match self {
            Self::Bool(b) => {
                if b {
                    1.0
                } else {
                    0.0
                }
            }
            Self::Number(n) => n,
        }
    }
}

impl From<bool> for Value {
    #[inline(always)]
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<f64> for Value {
    #[inline(always)]
    fn from(n: f64) -> Self {
        Self::Number(n)
    }
}

/// Context for expression evaluation.
pub struct ExpressionContext<'a> {
    pub timestamp: u64,
    pub entity_id: u32,
    variables: &'a [f64],
    property_getter: Option<&'a dyn Fn(&str) -> f64>,
    sensor_getter: Option<&'a dyn Fn(&str) -> bool>,
}

impl<'a> ExpressionContext<'a> {
    #[inline(always)]
    pub fn new(timestamp: u64, entity_id: u32) -> Self {
        Self {
            timestamp,
            entity_id,
            variables: &[],
            property_getter: None,
            sensor_getter: None,
        }
    }

    #[inline(always)]
    pub fn with_variables(mut self, variables: &'a [f64]) -> Self {
        self.variables = variables;
        self
    }

    #[inline(always)]
    pub fn with_property_getter(mut self, getter: &'a dyn Fn(&str) -> f64) -> Self {
        self.property_getter = Some(getter);
        self
    }

    #[inline(always)]
    pub fn with_sensor_getter(mut self, getter: &'a dyn Fn(&str) -> bool) -> Self {
        self.sensor_getter = Some(getter);
        self
    }

    #[inline(always)]
    pub fn get_var(&self, index: usize) -> f64 {
        self.variables.get(index).copied().unwrap_or(0.0)
    }

    #[inline(always)]
    pub fn get_property(&self, name: &str) -> f64 {
        self.property_getter.map(|g| g(name)).unwrap_or(0.0)
    }

    #[inline(always)]
    pub fn get_sensor(&self, name: &str) -> bool {
        self.sensor_getter.map(|g| g(name)).unwrap_or(false)
    }
}

/// Errors during expression evaluation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpressionError {
    StackUnderflow,
    UnknownOpCode(u8),
    DivisionByZero,
    InvalidOperand,
    NotCompiled,
}

/// Stack-based virtual machine for expression evaluation.
pub struct ExpressionVM {
    stack: Vec<Value>,
    pc: usize,
}

impl ExpressionVM {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            pc: 0,
        }
    }

    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            stack: Vec::with_capacity(capacity),
            pc: 0,
        }
    }

    #[inline(always)]
    pub fn reset(&mut self) {
        self.stack.clear();
        self.pc = 0;
    }

    pub fn execute(
        &mut self,
        bytecode: &[u8],
        constants: &[f64],
        context: &ExpressionContext,
    ) -> Result<bool, ExpressionError> {
        self.reset();

        while self.pc < bytecode.len() {
            let opcode = bytecode[self.pc];
            self.pc += 1;

            match opcode {
                0 => {} // Nop
                1 => {
                    // PushConst
                    let idx = bytecode[self.pc] as usize;
                    self.pc += 1;
                    if idx < constants.len() {
                        self.stack.push(Value::Number(constants[idx]));
                    } else {
                        return Err(ExpressionError::InvalidOperand);
                    }
                }
                2 => {
                    // LoadVar
                    let idx = bytecode[self.pc] as usize;
                    self.pc += 1;
                    self.stack.push(Value::Number(context.get_var(idx)));
                }
                3 => {
                    let _ = bytecode[self.pc];
                    self.pc += 1;
                } // StoreVar (noop)
                10 => self.binary_op(|a, b| Ok(Value::Number(a + b)))?, // Add
                11 => self.binary_op(|a, b| Ok(Value::Number(a - b)))?, // Subtract
                12 => self.binary_op(|a, b| Ok(Value::Number(a * b)))?, // Multiply
                13 => self.binary_op(|a, b| {
                    if b == 0.0 {
                        Err(ExpressionError::DivisionByZero)
                    } else {
                        Ok(Value::Number(a / b))
                    }
                })?, // Divide
                14 => self.binary_op(|a, b| Ok(Value::Number(a % b)))?, // Modulo
                15 => {
                    let a = self.stack.pop().ok_or(ExpressionError::StackUnderflow)?;
                    self.stack.push(Value::Number(-a.as_number()));
                } // Negate
                20 => self.binary_op(|a, b| Ok(Value::Bool(a == b)))?,  // Equal
                21 => self.binary_op(|a, b| Ok(Value::Bool(a != b)))?,  // NotEqual
                22 => self.binary_op(|a, b| Ok(Value::Bool(a > b)))?,   // Greater
                23 => self.binary_op(|a, b| Ok(Value::Bool(a >= b)))?,  // GreaterEqual
                24 => self.binary_op(|a, b| Ok(Value::Bool(a < b)))?,   // Less
                25 => self.binary_op(|a, b| Ok(Value::Bool(a <= b)))?,  // LessEqual
                30 => self.binary_op(|a, b| Ok(Value::Bool(a != 0.0 && b != 0.0)))?, // And
                31 => self.binary_op(|a, b| Ok(Value::Bool(a != 0.0 || b != 0.0)))?, // Or
                32 => {
                    let a = self.stack.pop().ok_or(ExpressionError::StackUnderflow)?;
                    self.stack.push(Value::Bool(!a.as_bool()));
                } // Not
                40 => {
                    // Jump
                    self.pc =
                        ((bytecode[self.pc] as usize) << 8) | (bytecode[self.pc + 1] as usize);
                }
                41 => {
                    // JumpIfFalse
                    let addr =
                        ((bytecode[self.pc] as usize) << 8) | (bytecode[self.pc + 1] as usize);
                    self.pc += 2;
                    let cond = self.stack.pop().ok_or(ExpressionError::StackUnderflow)?;
                    if !cond.as_bool() {
                        self.pc = addr;
                    }
                }
                42 => {
                    // JumpIfTrue
                    let addr =
                        ((bytecode[self.pc] as usize) << 8) | (bytecode[self.pc + 1] as usize);
                    self.pc += 2;
                    let cond = self.stack.pop().ok_or(ExpressionError::StackUnderflow)?;
                    if cond.as_bool() {
                        self.pc = addr;
                    }
                }
                52 => self.stack.push(Value::Number(context.timestamp as f64)), // GetTimestamp
                53 => self.stack.push(Value::Number(context.entity_id as f64)), // GetEntityId
                _ => return Err(ExpressionError::UnknownOpCode(opcode)),
            }
        }

        Ok(self.stack.pop().map(|v| v.as_bool()).unwrap_or(false))
    }

    #[inline(always)]
    fn binary_op<F>(&mut self, op: F) -> Result<(), ExpressionError>
    where
        F: FnOnce(f64, f64) -> Result<Value, ExpressionError>,
    {
        let b = self.stack.pop().ok_or(ExpressionError::StackUnderflow)?;
        let a = self.stack.pop().ok_or(ExpressionError::StackUnderflow)?;
        self.stack.push(op(a.as_number(), b.as_number())?);
        Ok(())
    }
}

impl Default for ExpressionVM {
    fn default() -> Self {
        Self::new()
    }
}

/// Evaluates an expression with the given bytecode and context.
#[inline(always)]
pub fn evaluate_expression(
    bytecode: &[u8],
    constants: &[f64],
    context: &ExpressionContext,
) -> Result<bool, ExpressionError> {
    let mut vm = ExpressionVM::new();
    vm.execute(bytecode, constants, context)
}

/// Compiles an expression string into bytecode.
pub fn compile_expression(source: &str) -> Result<CompiledExpression, ExpressionError> {
    let mut bytecode = Vec::new();
    let mut constants = Vec::new();
    let mut variables = Vec::new();

    let parts: Vec<&str> = source.split_whitespace().collect();

    if parts.len() >= 3 {
        let var_name = parts[0].to_string();
        let op = parts[1];
        let const_str = parts[2];

        let var_idx = variables.len();
        variables.push(var_name);

        if let Ok(val) = const_str.parse::<f64>() {
            let const_idx = constants.len();
            constants.push(val);

            bytecode.push(2); // LoadVar
            bytecode.push(var_idx as u8);

            bytecode.push(1); // PushConst
            bytecode.push(const_idx as u8);

            match op {
                "==" => bytecode.push(20),
                "!=" => bytecode.push(21),
                ">" => bytecode.push(22),
                ">=" => bytecode.push(23),
                "<" => bytecode.push(24),
                "<=" => bytecode.push(25),
                "+" => bytecode.push(10),
                "-" => bytecode.push(11),
                "*" => bytecode.push(12),
                "/" => bytecode.push(13),
                "%" => bytecode.push(14),
                "&&" | "AND" => bytecode.push(30),
                "||" | "OR" => bytecode.push(31),
                _ => return Err(ExpressionError::InvalidOperand),
            }
        }
    }

    Ok(CompiledExpression::new(bytecode, constants, variables, 4))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expression_vm_simple() {
        let mut vm = ExpressionVM::new();
        let context = ExpressionContext::new(1000, 1);
        let bytecode = vec![1, 0, 1, 1, 22]; // 5 > 3
        let constants = vec![5.0, 3.0];
        assert!(vm.execute(&bytecode, &constants, &context).unwrap());
    }

    #[test]
    fn test_expression_vm_and() {
        let mut vm = ExpressionVM::new();
        let context = ExpressionContext::new(1000, 1);
        let bytecode = vec![1, 0, 1, 1, 30]; // true AND false
        let constants = vec![1.0, 0.0];
        assert!(!vm.execute(&bytecode, &constants, &context).unwrap());
    }

    #[test]
    fn test_expression_vm_not() {
        let mut vm = ExpressionVM::new();
        let context = ExpressionContext::new(1000, 1);
        let bytecode = vec![1, 0, 32]; // NOT true
        let constants = vec![1.0];
        assert!(!vm.execute(&bytecode, &constants, &context).unwrap());
    }

    #[test]
    fn test_expression_vm_variables() {
        let mut vm = ExpressionVM::new();
        let context = ExpressionContext::new(1000, 1).with_variables(&[10.0, 5.0]);
        let bytecode = vec![2, 0, 2, 1, 22]; // var[0] > var[1]
        let constants = vec![];
        assert!(vm.execute(&bytecode, &constants, &context).unwrap());
    }

    #[test]
    fn test_compile_simple_expression() {
        let result = compile_expression("x > 5");
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_expression_equality() {
        let result = compile_expression("value == 10");
        assert!(result.is_ok());
        assert!(result.unwrap().constants.contains(&10.0));
    }

    #[test]
    fn test_compile_expression_logical() {
        assert!(compile_expression("x AND y").is_ok());
        assert!(compile_expression("a || b").is_ok());
    }

    #[test]
    fn test_evaluate_expression_function() {
        let bytecode = vec![1, 0, 1, 1, 24]; // 3 < 5
        let constants = vec![3.0, 5.0];
        let context = ExpressionContext::new(1000, 1);
        assert!(evaluate_expression(&bytecode, &constants, &context).unwrap());
    }
}
