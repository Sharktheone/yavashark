use std::cell::RefCell;
use yavashark_macro::{object, properties_new};
use yavashark_value::Obj;
use crate::{object, Error, MutObject, Object, ObjectHandle, Realm, Result, Value};
use crate::array::Array;

#[object]
#[derive(Debug)]
pub struct JSON {}


impl JSON {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(proto: ObjectHandle, func: ObjectHandle) -> Result<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableJSON {
                object: MutObject::with_proto(proto.into()),
            }),
        };

        this.initialize(func.into())?;

        Ok(this.into_object())
    }


    fn value_from_serde(value: serde_json::Value, realm: &Realm) -> Result<Value> {
        Ok(match value {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(b) => Value::Boolean(b),
            serde_json::Value::Number(n) => Value::Number(n.as_f64().unwrap_or(0.0)),
            serde_json::Value::String(s) => Value::String(s),
            serde_json::Value::Array(a) => {
                let values = a.into_iter().map(|v| Self::value_from_serde(v, realm)).collect::<Result<Vec<_>>>()?;

                Array::with_elements(realm, values)?.into_value()
            }
            serde_json::Value::Object(o) => {
                let mut obj = MutObject::new(realm);

                for (k, v) in o.into_iter() {
                    let val = Self::value_from_serde(v, realm)?;

                    obj.define_property(k.into(), val)?
                }

                Object::from_mut(obj).into_value()
            }

        })
    }

}


#[properties_new(raw)]
impl JSON {
    fn parse(str: String, #[realm] realm: &Realm) -> Result<Value> {
        let value: serde_json::Value = match serde_json::from_str(&str) {
            Ok(value) => value,
            Err(error) => {
                return Err(Error::syn_error(error.to_string()))
            }
        };


        Self::value_from_serde(value, realm)
    }



}
