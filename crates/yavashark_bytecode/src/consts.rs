use std::rc::Rc;
use crate::data::DataSection;
use crate::instructions;
use num_bigint::BigInt;
use yavashark_value::ConstString;
use swc_ecma_ast::Param;

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
pub struct ObjectLiteralBlueprint {
    pub properties: Vec<(String, ConstValue)>,
}



#[derive(Debug, Clone, PartialEq)]
pub struct ArrayLiteralBlueprint {
    pub properties: Vec<ConstValue>,
}


#[derive(Debug, PartialEq, Clone)]
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
