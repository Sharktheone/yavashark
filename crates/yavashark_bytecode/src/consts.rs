use crate::data::DataSection;
use crate::function::BytecodeFunction;
use crate::Instruction;
use yavashark_env::{Value};
use yavashark_env::realm::Realm;
use yavashark_value::ConstString;

#[derive(Debug, Clone, PartialEq)]
pub enum ConstValue {
    Null,
    Undefined,
    Number(f64),
    String(String),
    Boolean(bool),
    Object(ObjectLiteralBlueprint),
    Function(FunctionBlueprint),
    Symbol(ConstString),
}

impl ConstValue {
    #[must_use]
    pub fn into_value(self, realm: &Realm) -> Value {
        match self {
            Self::Null => Value::Null,
            Self::Undefined => Value::Undefined,
            Self::Number(n) => Value::Number(n),
            Self::String(s) => Value::String(s),
            Self::Boolean(b) => Value::Boolean(b),
            Self::Object(_) => todo!(),
            Self::Symbol(s) => Value::Symbol(s),
            Self::Function(f) => BytecodeFunction::from_blueprint(f, realm).into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectLiteralBlueprint;

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionBlueprint {
    pub name: Option<String>,
    pub params: Vec<String>,
    pub body: Vec<Instruction>,
    pub data: DataSection,
}
