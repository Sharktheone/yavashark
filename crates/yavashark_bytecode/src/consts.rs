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
    pub fn into_value(self) -> Value {
        match self {
            ConstValue::Null => Value::Null,
            ConstValue::Undefined => Value::Undefined,
            ConstValue::Number(n) => Value::Number(n),
            ConstValue::String(s) => Value::String(s),
            ConstValue::Boolean(b) => Value::Boolean(b),
            ConstValue::Object(_) => todo!(),
            ConstValue::Symbol(s) => Value::Symbol(s),
        }
    }
}


#[derive(Debug, Clone)]
pub struct ObjectLiteralBlueprint {}
