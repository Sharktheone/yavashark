use std::cell::RefCell;
use yavashark_bytecode::{ArrayLiteralBlueprint, ConstValue, ObjectLiteralBlueprint};
use yavashark_env::{Object, Realm, Value, ValueResult};
use yavashark_env::array::Array;
use yavashark_env::builtins::RegExp;
use yavashark_env::optimizer::{FunctionCode, OptimFunction};
use yavashark_env::scope::Scope;
use yavashark_env::value::Obj;
use crate::function_code::BytecodeFunction;

pub trait ConstIntoValue {
    fn into_value(self, realm: &Realm, scope: &Scope) -> ValueResult;
}

impl ConstIntoValue for ConstValue {
    fn into_value(self, realm: &Realm, scope: &Scope) -> ValueResult {
        Ok(match self {
            Self::Null => Value::Null,
            Self::Undefined => Value::Undefined,
            Self::Number(n) => Value::Number(n),
            Self::String(s) => Value::String(s),
            Self::Boolean(b) => Value::Boolean(b),
            Self::Object(obj) => obj.into_value(realm, scope)?,
            Self::Array(array) => array.into_value(realm, scope)?,
            Self::Symbol(s) => Value::Symbol(s.into()),
            Self::Function(bp) => {

                let func: RefCell<Box<dyn FunctionCode>> = RefCell::new(Box::new(BytecodeFunction {
                    code: bp.code,
                    is_async: bp.is_async,
                    is_generator: bp.is_generator,
                }));
                
                let optim = OptimFunction::new(
                    bp.name.unwrap_or("anonymous".to_string()),
                    bp.params,
                    Some(func),
                    scope.clone(),
                    realm,
                )?;
                
                optim.into()
            },
            Self::BigInt(b) => Value::BigInt(b),
            Self::Regex(exp, flags) => {
                RegExp::new_from_str_with_flags(realm, &exp, &flags)?.into()
            }
        })
    }
}

impl ConstIntoValue for ArrayLiteralBlueprint {
    fn into_value(self, realm: &Realm, scope: &Scope) -> ValueResult {
        let props = self
            .properties
            .into_iter()
            .map(|v| v.into_value(realm, scope))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Array::with_elements(realm, props)?.into_value())
    }
}

impl ConstIntoValue for ObjectLiteralBlueprint {
    fn into_value(self, realm: &Realm, scope: &Scope) -> ValueResult {
        let obj = Object::new(realm);

        for (key, value) in self.properties {
            obj.define_property(key.into(), value.into_value(realm, scope)?)?;
        }

        Ok(obj.into())
    }
}
