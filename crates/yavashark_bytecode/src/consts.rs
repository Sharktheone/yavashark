use crate::data::{Acc, DataSection, OutputDataType, Reg, Stack, VarName};
use crate::instructions;
use num_bigint::BigInt;
use std::rc::Rc;
use swc_ecma_ast::Param;
use yavashark_value::ConstString;

#[derive(Debug, Clone, PartialEq)]
pub enum ConstValue {
    Null,
    Undefined,
    Number(f64),
    String(String),
    Boolean(bool),
    Object(ObjectLiteralBlueprint),
    Array(ArrayLiteralBlueprint),
    Function(FunctionBlueprint),
    BigInt(BigInt),
    Regex(String, String),
    Symbol(ConstString),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataTypeValue {
    Null,
    Undefined,
    Number(f64),
    String(String),
    Boolean(bool),
    Object(ObjectLiteralBlueprint),
    Array(ArrayLiteralBlueprint),
    Function(FunctionBlueprint),
    BigInt(BigInt),
    Regex(String, String),
    Symbol(ConstString),

    Acc(Acc),
    Reg(Reg),
    Var(VarName),
    Stack(Stack),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectLiteralBlueprint {
    pub properties: Vec<(DataTypeValue, DataTypeValue)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayLiteralBlueprint {
    pub properties: Vec<Option<DataTypeValue>>,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct BytecodeFunctionCode {
    pub instructions: Vec<instructions::Instruction>,
    pub ds: DataSection,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionBlueprint {
    pub name: Option<String>,
    pub params: Vec<Param>,
    pub code: Rc<BytecodeFunctionCode>,
    pub is_async: bool,
    pub is_generator: bool,
}

impl From<&str> for ConstValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<String> for ConstValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<OutputDataType> for DataTypeValue {
    fn from(val: OutputDataType) -> Self {
        match val {
            OutputDataType::Acc(acc) => Self::Acc(acc),
            OutputDataType::Reg(reg) => Self::Reg(reg),
            OutputDataType::Var(variable) => Self::Var(variable),
            OutputDataType::Stack(stack) => Self::Stack(stack),
        }
    }
}

impl From<ConstValue> for DataTypeValue {
    fn from(val: ConstValue) -> Self {
        match val {
            ConstValue::Null => Self::Null,
            ConstValue::Undefined => Self::Undefined,
            ConstValue::Number(n) => Self::Number(n),
            ConstValue::String(s) => Self::String(s),
            ConstValue::Boolean(b) => Self::Boolean(b),
            ConstValue::Object(obj) => Self::Object(obj),
            ConstValue::Array(arr) => Self::Array(arr),
            ConstValue::Function(func) => Self::Function(func),
            ConstValue::BigInt(b) => Self::BigInt(b),
            ConstValue::Regex(exp, flags) => Self::Regex(exp, flags),
            ConstValue::Symbol(s) => Self::Symbol(s),
        }
    }
}
