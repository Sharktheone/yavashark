use crate::array::Array;
use crate::builtins::{BigIntObj, BooleanObj, NumberObj, StringObj, SymbolObj};
use crate::object::common;
use crate::{MutObject, Object, ObjectHandle, Realm, Result, Value, ValueResult, Variable};
use std::cell::RefCell;
use std::mem;
use yavashark_macro::{object, properties_new};
use yavashark_value::{Constructor, Func, Obj};

#[object(constructor, function)]
#[derive(Debug)]
pub struct ObjectConstructor {}

impl Constructor<Realm> for ObjectConstructor {
    fn construct(&self, realm: &mut Realm, mut args: Vec<Value>) -> ValueResult {
        let Some(value) = args.first_mut() else {
            return Ok(Object::new(realm).into());
        };

        let value = mem::replace(value, Value::Undefined);

        Ok(match value {
            Value::Object(obj) => obj.into(),
            Value::Number(num) => NumberObj::with_number(realm, num)?.into(),
            Value::String(string) => StringObj::with_string(realm, string)?.into(),
            Value::Boolean(boolean) => BooleanObj::new(realm, boolean).into(),
            Value::Symbol(symbol) => SymbolObj::new(realm, symbol).into(),
            Value::BigInt(bigint) => BigIntObj::new(realm, bigint).into(),
            Value::Undefined | Value::Null => Object::new(realm).into(),
        })
    }
}

impl Func<Realm> for ObjectConstructor {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, _: Value) -> ValueResult {
        Constructor::construct(self, realm, args)
    }
}

impl ObjectConstructor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(proto: Value, func: Value) -> Result<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableObjectConstructor {
                object: MutObject::with_proto(proto),
            }),
        };

        this.initialize(func)?;

        Ok(this.into_object())
    }
}

#[properties_new(raw)]
impl ObjectConstructor {
    fn create(proto: ObjectHandle) -> ObjectHandle {
        Object::with_proto(proto.into())
    }

    #[prop("defineProperty")]
    pub fn define_property(obj: ObjectHandle, key: Value, descriptor: &ObjectHandle) -> ValueResult {
        let value = descriptor
            .get_property(&"value".into())
            .map(|v| v.value)
            .unwrap_or(Value::Undefined);

        let writable = descriptor
            .get_property(&"writable".into())
            .map(|v| v.value.is_truthy())
            .unwrap_or(false);
        let enumerable = descriptor
            .get_property(&"enumerable".into())
            .map(|v| v.value.is_truthy())
            .unwrap_or(false);
        let configurable = descriptor
            .get_property(&"configurable".into())
            .map(|v| v.value.is_truthy())
            .unwrap_or(false);

        let var = Variable::new_with_attributes(value, writable, enumerable, configurable);

        obj.define_variable(key, var)?;

        Ok(obj.into())
    }

    fn assign(
        target: ObjectHandle,
        #[variadic] sources: &[Value],
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        for source in sources {
            let source = source.as_object()?;

            for key in source.keys()? {
                let value = source.get_property(&key)?;

                if !value.attributes.is_enumerable() {
                    continue;
                }

                let value = value.resolve(source.clone().into(), realm)?;

                target.define_property(key, value)?;
            }
        }

        Ok(target.into())
    }

    #[prop("defineProperties")]
    fn define_properties(obj: ObjectHandle, props: &ObjectHandle) -> ValueResult {
        for (key, value) in props.properties()? {
            let descriptor = value.as_object()?;

            Self::define_property(obj.clone(), key, descriptor)?;
        }

        Ok(obj.into())
    }

    #[prop("entries")]
    fn entries(obj: &ObjectHandle, #[realm] realm: &mut Realm) -> ValueResult {
        let keys = obj.keys()?;

        let mut props = Vec::with_capacity(keys.len());

        for key in keys {
            let value = obj.get_property(&key)?;

            if !value.attributes.is_enumerable() {
                continue;
            }

            let value = value.resolve(obj.clone().into(), realm)?;

            let arr = vec![key, value];

            let arr = Array::with_elements(realm, arr)?;

            props.push(arr.into_value());
        }

        Ok(Array::with_elements(realm, props)?.into_value())
    }

    #[prop("getOwnPropertyDescriptor")]
    fn get_own_property_descriptor(
        #[this] this: Value,
        #[variadic] args: &[Value],
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        common::get_own_property_descriptor(args.to_vec(), this, realm)
    }

    #[prop("getOwnPropertyDescriptors")]
    fn get_own_property_descriptors(obj: &ObjectHandle, #[realm] realm: &mut Realm) -> ValueResult {
        let keys = obj.keys()?;

        let mut props = Vec::with_capacity(keys.len());

        for key in keys {
            let value = obj.get_property(&key)?;

            if !value.attributes.is_enumerable() {
                continue;
            }

            let desc =
                common::get_own_property_descriptor(vec![key.clone()], obj.clone().into(), realm)?;

            props.push((key, desc));
        }

        Ok(Object::from_values(props, realm)?.into())
    }

    #[prop("getOwnPropertyNames")]
    fn get_own_property_names(obj: &ObjectHandle, #[realm] realm: &Realm) -> ValueResult {
        let keys = obj.keys()?;

        let mut props = Vec::with_capacity(keys.len());

        for key in keys {
            props.push(key);
        }

        Ok(Array::with_elements(realm, props)?.into_value())
    }

    #[prop("getOwnPropertySymbols")]
    fn get_own_property_symbols(obj: &ObjectHandle, #[realm] realm: &Realm) -> ValueResult {
        let keys = obj.keys()?;

        let mut props = Vec::with_capacity(keys.len());

        for key in keys {
            if !matches!(key, Value::Symbol(_)) {
                continue;
            }

            // if !value.attributes.is_enumerable() { // TODO: not sure
            //     continue;
            // }

            props.push(key);
        }

        Ok(Array::with_elements(realm, props)?.into_value())
    }

    #[prop("getPrototypeOf")]
    fn get_prototype_of(val: Value, #[realm] realm: &mut Realm) -> ValueResult {
        let Value::Object(ref obj) = val else {
            return Ok(Object::new(realm).into());
        };

        let prop = obj.prototype()?;

        prop.resolve(val, realm)
    }

    #[prop("groupBy")]
    fn group_by(_items: ObjectHandle, _callback: Value, #[realm] realm: &Realm) -> ValueResult {
        //TODO

        Ok(Object::new(realm).into())
    }

    #[prop("hasOwn")]
    fn has_own(obj: &ObjectHandle, key: &Value) -> ValueResult {
        Ok(obj.contains_key(key)?.into())
    }

    #[prop("is")]
    fn is(val1: &Value, val2: &Value) -> ValueResult {
        //TODO: handle things like NaN, -0, etc. BigInt & Numbers

        Ok((val1 == val2).into())
    }

    #[prop("keys")]
    fn keys(obj: &ObjectHandle, #[realm] realm: &Realm) -> ValueResult {
        let keys = obj.keys()?;

        Ok(Array::with_elements(realm, keys)?.into_value())
    }

    #[prop("values")]
    fn values(obj: &ObjectHandle, #[realm] realm: &mut Realm) -> ValueResult {
        let keys = obj.keys()?;

        let mut props = Vec::with_capacity(keys.len());

        for key in keys {
            let value = obj.get_property(&key)?;

            if !value.attributes.is_enumerable() {
                continue;
            }

            let value = value.resolve(obj.clone().into(), realm)?;

            props.push(value);
        }

        Ok(Array::with_elements(realm, props)?.into_value())
    }

    #[prop("setPrototypeOf")]
    fn set_prototype_of(obj: ObjectHandle, proto: Value) -> ValueResult {
        obj.define_property("__proto__".into(), proto)?;

        Ok(obj.into())
    }
}
