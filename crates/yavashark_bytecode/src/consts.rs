use yavashark_env::Value;
use yavashark_value::ConstString;

#[derive(Debug, Clone)]
pub enum ConstValue {
    Null,
    Undefined,
    Number(f64),
    String(String),
    Boolean(bool),
    Object(ObjectLiteralBlueprint),
    Symbol(ConstString),
}

impl ConstValue {
    #[must_use] pub fn into_value(self) -> Value {
        match self {
            Self::Null => Value::Null,
            Self::Undefined => Value::Undefined,
            Self::Number(n) => Value::Number(n),
            Self::String(s) => Value::String(s),
            Self::Boolean(b) => Value::Boolean(b),
            Self::Object(_) => todo!(),
            Self::Symbol(s) => Value::Symbol(s),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ObjectLiteralBlueprint;
