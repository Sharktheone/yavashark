use crate::array::Array;
use crate::builtins::{BigIntObj, BooleanObj, NumberObj, StringObj, SymbolObj};
use crate::object::common;
use crate::{Error, MutObject, Object, ObjectHandle, Realm, Res, Value, ValueResult, Variable};
use std::cell::RefCell;
use std::collections::HashMap;
use std::mem;
use yavashark_macro::{object, properties_new};
use yavashark_value::{Constructor, Func, IntoValue, Obj};

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
            Value::String(string) => Obj::into_value(StringObj::with_string(realm, string)),
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
    pub fn new(proto: Value, func: Value) -> Res<ObjectHandle> {
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
    fn create(proto: Value, properties: Option<ObjectHandle>) -> Res<ObjectHandle> {
        if !proto.is_object() && !proto.is_null() {
            return Err(Error::ty("Object prototype must be an object or null"));
        }

        let obj = Object::with_proto(proto);

        if let Some(props) = properties {
            for (key, value) in props.properties()? {
                Self::define_property(obj.clone(), &key, value.as_object()?)?;
            }
        }

        Ok(obj)
    }

    #[prop("defineProperty")]
    pub fn define_property(
        obj: ObjectHandle,
        key: &Value,
        descriptor: &ObjectHandle,
    ) -> ValueResult {
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

        obj.define_variable(key.copy(), var)?;

        //TODO: there should be a obj.define_property which takes a descriptor
        if let Some(get) = descriptor.resolve_property_no_get_set(&"get".into())? {
            obj.define_getter(key.copy(), get.value)?;
        }

        if let Some(set) = descriptor.resolve_property_no_get_set(&"set".into())? {
            obj.define_setter(key.copy(), set.value)?;
        }

        Ok(obj.into())
    }

    fn assign(
        target: Value,
        #[variadic] sources: &[Value],
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let target = match target {
            Value::Object(obj) => obj,
            Value::Undefined | Value::Null => {
                return Err(Error::ty("Cannot assign to undefined or null"))
            }
            Value::Boolean(b) => BooleanObj::new(realm, b),
            Value::Number(n) => NumberObj::with_number(realm, n)?,
            Value::String(s) => StringObj::with_string(realm, s).into_object(),
            Value::Symbol(s) => SymbolObj::new(realm, s),
            Value::BigInt(b) => BigIntObj::new(realm, b),
        };

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
    fn define_properties(obj: ObjectHandle, props: &Value) -> ValueResult {
        let Ok(props) = props.as_object() else {
            return Ok(obj.into());
        };

        for (key, value) in props.properties()? {
            let descriptor = value.as_object()?;

            Self::define_property(obj.clone(), &key, descriptor)?;
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
        let mut keys = obj.keys()?;

        keys.retain(|k| !k.is_symbol());

        Ok(Array::with_elements(realm, keys)?.into_value())
    }

    #[prop("getOwnPropertySymbols")]
    fn get_own_property_symbols(obj: &ObjectHandle, #[realm] realm: &Realm) -> ValueResult {
        let mut keys = obj.keys()?;

        keys.retain(Value::is_symbol);

        Ok(Array::with_elements(realm, keys)?.into_value())
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
    fn group_by(items: Value, callback: Value, #[realm] realm: &mut Realm) -> ValueResult {
        let iter = items.iter_no_realm(realm)?;

        let mut groups = HashMap::new();

        while let Some(item) = iter.next(realm)? {
            let key = callback.call(realm, vec![item.clone()], Value::Undefined)?;

            let key = key.to_string(realm)?;

            groups.entry(key).or_insert_with(Vec::new).push(item);
        }

        let result = Object::new(realm);

        for (key, values) in groups {
            let arr = Array::with_elements(realm, values)?;
            result.define_property(key.into(), arr.into_value())?;
        }

        Ok(result.into_value())
    }

    #[prop("hasOwn")]
    fn has_own(obj: &ObjectHandle, key: &Value) -> ValueResult {
        Ok(obj.contains_key(key)?.into())
    }

    #[prop("is")]
    fn is(val1: &Value, val2: &Value) -> ValueResult {
        match (val1, val2) {
            (Value::Number(n1), Value::Number(n2)) => {
                if n1.is_nan() && n2.is_nan() {
                    return Ok(Value::Boolean(true));
                }

                if n1.is_sign_positive() && n2.is_sign_negative() {
                    return Ok(Value::Boolean(false));
                }

                if n1.is_sign_negative() && n2.is_sign_positive() {
                    return Ok(Value::Boolean(false));
                }
            }

            _ => {}
        }

        Ok((val1 == val2).into())
    }

    #[prop("keys")]
    pub fn keys_js(obj: &Value, #[realm] realm: &Realm) -> ValueResult {
        let Value::Object(obj) = obj else {
            return Ok(Array::from_realm(realm).into_value());
        };

        let keys = obj
            .keys()?
            .iter()
            .filter_map(|k| {
                let v = obj.get_property(k).ok()?; //TODO: This is absolutely not how this should be done (performance wise)

                if v.attributes.is_enumerable() {
                    Some(k.copy())
                } else {
                    None
                }
            })
            .collect();

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
    fn set_prototype_of(obj: Value, proto: Value, #[realm] realm: &mut Realm) -> ValueResult {
        if obj.instance_of(&proto, realm)? {
            return Err(Error::ty("Cannot set prototype to itself"));
        }

        obj.as_object()?.set_prototype(proto.into())?;

        Ok(obj)
    }

    #[prop("preventExtensions")]
    const fn prevent_extensions(_: &ObjectHandle) {}

    #[prop("isExtensible")]
    const fn is_extensible(_: &ObjectHandle) -> bool {
        true
    }
    
    #[prop("seal")]
    const fn seal(_: &ObjectHandle) {}
}
