use crate::array::Array;
use crate::{Error, MutObject, Object, ObjectHandle, Realm, Res, Value};
use serde_json::{Map, Number};
use std::cell::RefCell;
use yavashark_macro::{object, properties_new};
use yavashark_value::Obj;

#[object]
#[derive(Debug)]
pub struct JSON {}

impl JSON {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(proto: ObjectHandle, func: ObjectHandle) -> Res<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableJSON {
                object: MutObject::with_proto(proto.into()),
            }),
        };

        this.initialize(func.into())?;

        Ok(this.into_object())
    }

    fn value_from_serde(value: serde_json::Value, realm: &Realm) -> Res<Value> {
        Ok(match value {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(b) => Value::Boolean(b),
            serde_json::Value::Number(n) => Value::Number(n.as_f64().unwrap_or(0.0)),
            serde_json::Value::String(s) => Value::String(s.into()),
            serde_json::Value::Array(a) => {
                let values = a
                    .into_iter()
                    .map(|v| Self::value_from_serde(v, realm))
                    .collect::<Res<Vec<_>>>()?;

                Array::with_elements(realm, values)?.into_value()
            }
            serde_json::Value::Object(o) => {
                let mut obj = MutObject::new(realm);

                for (k, v) in o {
                    let val = Self::value_from_serde(v, realm)?;

                    obj.define_property(k.into(), val)?;
                }

                Object::from_mut(obj).into_value()
            }
        })
    }

    fn value_to_serde(value: Value, realm: &mut Realm) -> Res<Option<serde_json::Value>> {
        //TODO: handle circular items
        Ok(Some(match value {
            Value::Null => serde_json::Value::Null,
            Value::Undefined | Value::Symbol(_) => return Ok(None),
            Value::Boolean(b) => serde_json::Value::Bool(b),
            Value::Number(n) => {
                serde_json::Value::Number(Number::from_f64(n).unwrap_or(Number::from(0u8)))
            }
            Value::String(s) => serde_json::Value::String(s.to_string()),
            Value::BigInt(_) => return Err(Error::ty("Do not know how to serialize a BigInt")),
            Value::Object(ref o) => {
                if value.instance_of(&realm.intrinsics.array_constructor().value, realm)? {
                    let mut index = 0;

                    let mut array = Vec::new();

                    loop {
                        let (done, value) = o.get_array_or_done(index)?;

                        if done {
                            break;
                        }

                        let val;

                        if let Some(value) = value {
                            val = Self::value_to_serde(value, realm)?
                                .unwrap_or(serde_json::Value::Null);
                        } else {
                            val = serde_json::Value::Null;
                        }

                        array.push(val);

                        index += 1;
                    }

                    return Ok(Some(serde_json::Value::Array(array)));
                };

                let props = o.properties()?;
                //TODO: handle getters

                let mut map = Map::with_capacity(props.len());

                for (k, v) in props {
                    let Some(val) = Self::value_to_serde(v, realm)? else {
                        continue;
                    };

                    let k = k.to_string(realm)?;

                    map.insert(k.to_string(), val);
                }

                serde_json::Value::Object(map)
            }
        }))
    }
}

#[properties_new(raw)]
impl JSON {
    fn parse(str: &str, #[realm] realm: &Realm) -> Res<Value> {
        let value: serde_json::Value = match serde_json::from_str(str) {
            Ok(value) => value,
            Err(error) => return Err(Error::syn_error(error.to_string())),
        };

        Self::value_from_serde(value, realm)
    }

    fn stringify(value: Value, #[realm] realm: &mut Realm) -> Res<Value> {
        let value = Self::value_to_serde(value, realm)?;

        value.map_or_else(
            || Ok(Value::Undefined),
            |value| Ok(serde_json::to_string(&value).unwrap_or_default().into()),
        )
    }
}
