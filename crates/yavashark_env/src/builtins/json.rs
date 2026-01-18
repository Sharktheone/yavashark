use crate::array::Array;
use crate::partial_init::Initializer;
use crate::realm::Intrinsic;
use crate::value::{Hint, IntoValue, Obj};
use crate::{Error, MutObject, Object, ObjectHandle, Realm, Res, Value};
use serde_json::{Map, Number};
use std::cell::RefCell;
use yavashark_macro::{object, properties_new};

#[object]
#[derive(Debug)]
pub struct JSON {}

impl JSON {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(realm: &mut Realm) -> Res<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableJSON {
                object: MutObject::with_proto(realm.intrinsics.obj.clone()),
            }),
        };

        this.initialize(realm)?;

        Ok(this.into_object())
    }

    fn value_from_serde(value: serde_json::Value, realm: &mut Realm) -> Res<Value> {
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

                    obj.define_property(k.into(), val, realm)?;
                }

                Object::from_mut(obj).into_value()
            }
        })
    }

    fn value_to_serde(
        value: Value,
        realm: &mut Realm,
        visited: &mut Vec<usize>,
    ) -> Res<Option<serde_json::Value>> {
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
                if visited.contains(&o.as_ptr().addr()) {
                    return Err(Error::ty(
                        "Circular reference detected in JSON serialization",
                    ));
                }

                visited.push(o.as_ptr().addr());

                if value.instance_of(&Array::get_global(realm)?.into(), realm)? {
                    let mut index = 0;

                    let mut array = Vec::new();

                    loop {
                        let (done, value) = o.get_array_or_done(index, realm)?;

                        if done {
                            break;
                        }

                        let val;

                        if let Some(value) = value {
                            val = Self::value_to_serde(value, realm, visited)?
                                .unwrap_or(serde_json::Value::Null);
                        } else {
                            val = serde_json::Value::Null;
                        }

                        array.push(val);

                        index += 1;
                    }

                    return Ok(Some(serde_json::Value::Array(array)));
                }

                if let Some(prim) = o.primitive(realm)? {
                    return Self::value_to_serde(prim.into(), realm, visited);
                }

                let props = o.enum_properties(realm)?;
                //TODO: handle getters

                let mut map = Map::with_capacity(props.len());

                for (k, v) in props {
                    let Some(val) = Self::value_to_serde(v, realm, visited)? else {
                        continue;
                    };

                    let k = k.to_string();

                    map.insert(k.to_string(), val);
                }

                serde_json::Value::Object(map)
            }
        }))
    }
}

#[properties_new(raw)]
impl JSON {
    fn parse(str: &Value, #[realm] realm: &mut Realm) -> Res<Value> {
        let str = str.to_primitive(Hint::String, realm)?;
        let str = str.to_string(realm)?;

        let value: serde_json::Value = match serde_json::from_str(&str.as_str_lossy()) {
            Ok(value) => value,
            Err(error) => return Err(Error::syn_error(error.to_string())),
        };

        Self::value_from_serde(value, realm)
    }

    fn stringify(value: Value, #[realm] realm: &mut Realm) -> Res<Value> {
        let value = Self::value_to_serde(value, realm, &mut Vec::new())?;

        value.map_or_else(
            || Ok(Value::Undefined),
            |value| Ok(serde_json::to_string(&value).unwrap_or_default().into()),
        )
    }

    #[prop("isRawJSON")]
    fn is_raw_json(value: &Value, #[realm] realm: &mut Realm) -> Res<bool> {
        Ok(match value {
            Value::Object(obj) => obj.is_frozen() && obj.prototype(realm)?.is_null(),
            _ => false,
        })
    }

    #[prop("rawJSON")]
    fn raw_json(_str: &str) -> Res<ObjectHandle> {
        let obj = Object::null();

        obj.freeze()?;

        Ok(obj)
    }
}

impl Initializer<ObjectHandle> for JSON {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
        JSON::new(realm)
    }
}
