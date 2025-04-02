use crate::data::DataSection;
use crate::function::BytecodeFunction;
use crate::Instruction;
use num_bigint::BigInt;
use yavashark_env::array::Array;
use yavashark_env::builtins::RegExp;
use yavashark_env::realm::Realm;
use yavashark_env::{Object, Value, ValueResult};
use yavashark_value::{ConstString, IntoValue, Obj};

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

impl ConstValue {
    pub fn into_value(self, realm: &Realm) -> ValueResult {
        Ok(match self {
            Self::Null => Value::Null,
            Self::Undefined => Value::Undefined,
            Self::Number(n) => Value::Number(n),
            Self::String(s) => Value::String(s),
            Self::Boolean(b) => Value::Boolean(b),
            Self::Object(obj) => obj.into_value(realm)?,
            Self::Array(array) => array.into_value(realm)?,
            Self::Symbol(s) => Value::Symbol(s.into()),
            Self::Function(f) => BytecodeFunction::from_blueprint(f, realm).into(),
            Self::BigInt(b) => Value::BigInt(b),
            Self::Regex(exp, flags) => {
                RegExp::new_from_str_with_flags(realm, &exp, &flags)?.into_value()
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectLiteralBlueprint {
    pub properties: Vec<(String, ConstValue)>,
}

impl ObjectLiteralBlueprint {
    pub fn into_value(self, realm: &Realm) -> ValueResult {
        let obj = Object::new(realm);

        for (key, value) in self.properties {
            obj.define_property(key.into(), value.into_value(realm)?)?;
        }

        Ok(obj.into_value())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayLiteralBlueprint {
    pub properties: Vec<ConstValue>,
}

impl ArrayLiteralBlueprint {
    pub fn into_value(self, realm: &Realm) -> ValueResult {
        let props = self
            .properties
            .into_iter()
            .map(|v| v.into_value(realm))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Array::with_elements(realm, props)?.into_value())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionBlueprint {
    pub name: Option<String>,
    pub params: Vec<String>,
    pub body: Vec<Instruction>,
    pub data: DataSection,
}
