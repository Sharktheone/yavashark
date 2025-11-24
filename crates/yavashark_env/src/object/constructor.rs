use crate::array::Array;
use crate::builtins::{BigIntObj, BooleanObj, NumberObj, StringObj, SymbolObj};
use crate::object::prototype::common;
use crate::partial_init::Initializer;
use crate::utils::coerce_object;
use crate::value::property_key::IntoPropertyKey;
use crate::value::{Constructor, Func, IntoValue, Iter, Obj, ObjectOrNull, Property, PropertyDescriptor};
use crate::{
    Error, InternalPropertyKey, MutObject, Object, ObjectHandle, PropertyKey, Realm, Res, Value,
    ValueResult, Variable,
};
use indexmap::IndexMap;
use std::cell::RefCell;
use std::mem;
use yavashark_macro::{object, properties_new};
use crate::conversion::FromValueOutput;

#[object(constructor, function)]
#[derive(Debug)]
pub struct ObjectConstructor {}

impl Constructor for ObjectConstructor {
    fn construct(&self, realm: &mut Realm, mut args: Vec<Value>) -> Res<ObjectHandle> {
        let Some(value) = args.first_mut() else {
            return Ok(Object::new(realm).into());
        };

        let value = mem::replace(value, Value::Undefined);

        Ok(match value {
            Value::Object(obj) => obj,
            Value::Number(num) => NumberObj::with_number(realm, num)?,
            Value::String(string) => Obj::into_object(StringObj::with_string(realm, string)?),
            Value::Boolean(boolean) => BooleanObj::new(realm, boolean)?,
            Value::Symbol(symbol) => SymbolObj::new(realm, symbol)?,
            Value::BigInt(bigint) => BigIntObj::new(realm, bigint)?,
            Value::Undefined | Value::Null => Object::new(realm),
        })
    }
}

impl Func for ObjectConstructor {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, _: Value) -> ValueResult {
        Ok(Constructor::construct(self, realm, args)?.into())
    }
}

impl Initializer<ObjectHandle> for ObjectConstructor {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
        let ctor = ObjectConstructor::new(realm)?;

        ctor.define_property_attributes(
            "prototype".into(),
            Variable::new_read_only(realm.intrinsics.obj.clone().into()),
            realm,
        )?;

        Ok(ctor)
    }
}

impl ObjectConstructor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(realm: &mut Realm) -> Res<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableObjectConstructor {
                object: MutObject::with_proto(realm.intrinsics.obj.clone()),
            }),
        };

        this.initialize(realm)?;

        Ok(this.into_object())
    }
}

#[properties_new(raw)]
impl ObjectConstructor {
    fn create(
        proto: Value,
        properties: Option<ObjectHandle>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let proto: ObjectOrNull = proto.try_into()?;

        let obj = Object::with_proto(proto);

        if let Some(props) = properties {
            for (key, value) in props.properties(realm)? {
                if let Value::Object(value) = value {
                    Self::define_property(obj.clone(), key.into(), &value, realm)?;
                }
            }
        }

        Ok(obj)
    }

    #[prop("defineProperty")]
    pub fn define_property(
        obj: ObjectHandle,
        key: InternalPropertyKey,
        descriptor: PropertyDescriptor,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        obj.define_descriptor(key, descriptor, realm)?;
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
            Value::Boolean(b) => BooleanObj::new(realm, b)?,
            Value::Number(n) => NumberObj::with_number(realm, n)?,
            Value::String(s) => StringObj::with_string(realm, s)?.into_object(),
            Value::Symbol(s) => SymbolObj::new(realm, s)?,
            Value::BigInt(b) => BigIntObj::new(realm, b)?,
        };

        for source in sources {
            let source = coerce_object(source.clone(), realm)?;

            for key in source.keys(realm)? {
                let Some(value) = source.resolve_property_no_get_set(key.clone(), realm)? else {
                    continue;
                };

                if !value.attributes().is_enumerable() {
                    continue;
                }

                let value = match value {
                    Property::Value(v, _) => v,
                    Property::Getter(getter, _) => {
                        getter.call(Vec::new(), source.clone().into(), realm)?
                    }
                };

                target.define_property(key.into(), value, realm)?;
            }
        }

        Ok(target.into())
    }

    #[prop("defineProperties")]
    fn define_properties(
        obj: ObjectHandle,
        props: &Value,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let Ok(props) = props.as_object() else {
            return Ok(obj.into());
        };

        for (key, value) in props.properties(realm)? {
            let descriptor = value.as_object()?;

            Self::define_property(obj.clone(), key.into(), descriptor, realm)?;
        }

        Ok(obj.into())
    }

    #[prop("entries")]
    fn entries(obj: &ObjectHandle, #[realm] realm: &mut Realm) -> ValueResult {
        let keys = obj.keys(realm)?;

        let mut props = Vec::with_capacity(keys.len());

        for key in keys {
            let value = obj.resolve_property_no_get_set(key.clone(), realm)?;
            let Some(value) = value else {
                continue;
            };
            if !value.attributes().is_enumerable() {
                continue;
            }

            let value = match value {
                Property::Value(v, _) => v,
                Property::Getter(getter, _) => {
                    getter.call(Vec::new(), obj.clone().into(), realm)?
                }
            };

            let arr = vec![key.into(), value];

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
        let keys = obj.keys(realm)?;

        let mut props = Vec::with_capacity(keys.len());

        for key in keys {
            let Some(value) = obj.resolve_property_no_get_set(key.clone(), realm)? else {
                continue;
            };

            if !value.attributes().is_enumerable() {
                continue;
            }

            let desc = common::get_own_property_descriptor(
                vec![key.clone().into()],
                obj.clone().into(),
                realm,
            )?;

            props.push((key.into(), desc));
        }

        Ok(Object::from_values(props, realm)?.into())
    }

    #[prop("getOwnPropertyNames")]
    fn get_own_property_names(obj: &ObjectHandle, #[realm] realm: &mut Realm) -> ValueResult {
        let mut keys = obj.keys(realm)?;

        keys.retain(|k| !k.is_symbol());

        let keys = keys.into_iter().map(|k| k.into()).collect();

        Ok(Array::with_elements(realm, keys)?.into_value())
    }

    #[prop("getOwnPropertySymbols")]
    fn get_own_property_symbols(obj: &ObjectHandle, #[realm] realm: &mut Realm) -> ValueResult {
        let mut keys = obj.keys(realm)?;

        keys.retain(PropertyKey::is_symbol);

        let keys = keys.into_iter().map(|k| k.into()).collect();

        Ok(Array::with_elements(realm, keys)?.into_value())
    }

    #[prop("getPrototypeOf")]
    fn get_prototype_of(val: Value, #[realm] realm: &mut Realm) -> ValueResult {
        let Value::Object(ref obj) = val else {
            return Ok(Object::new(realm).into());
        };

        let prop = obj.prototype(realm)?;

        Ok(prop.into())
    }

    #[prop("groupBy")]
    fn group_by(items: Value, callback: Value, #[realm] realm: &mut Realm) -> ValueResult {
        let iter = items.iter_no_realm(realm)?;

        let mut groups = IndexMap::new();

        while let Some(item) = iter.next(realm)? {
            let key = callback.call(realm, vec![item.clone()], Value::Undefined)?;

            let key = key.to_string(realm)?;

            groups.entry(key).or_insert_with(Vec::new).push(item);
        }

        let result = Object::new(realm);

        for (key, values) in groups {
            let arr = Array::with_elements(realm, values)?;
            result.define_property(key.into(), arr.into_value(), realm)?;
        }

        Ok(result.into_value())
    }

    #[prop("fromEntries")]
    fn from_entries(entries: Value, #[realm] realm: &mut Realm) -> ValueResult {
        fn inner(iter: &Iter, realm: &mut Realm) -> ValueResult {
            let obj = Object::new(realm);

            while let Some(entry) = iter.next(realm)? {
                let entry = entry.as_object()?;

                let key = entry.get(0, realm)?;
                let value = entry.get(1, realm)?;

                obj.define_property(key.into_internal_property_key(realm)?, value, realm)?;
            }

            Ok(obj.into_value())
        }

        let iter = entries.iter_no_realm(realm)?;

        inner(&iter, realm).inspect_err(|_| {
            _ = iter.close(realm);
        })
    }

    #[prop("hasOwn")]
    fn has_own(
        obj: &ObjectHandle,
        key: InternalPropertyKey,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        Ok(obj.contains_own_key(key, realm)?.into())
    }

    #[prop("is")]
    fn is(val1: &Value, val2: &Value) -> ValueResult {
        if let (Value::Number(n1), Value::Number(n2)) = (val1, val2) {
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

        Ok((val1 == val2).into())
    }

    #[prop("keys")]
    pub fn keys_js(obj: &Value, #[realm] realm: &mut Realm) -> ValueResult {
        let obj = match obj {
            Value::Object(obj) => obj,
            Value::Undefined | Value::Null => {
                return Err(Error::ty("Object.keys() expects an object"))
            }
            _ => return Ok(Array::from_realm(realm)?.into_value()),
        };

        let keys = obj
            .enumerable_keys(realm)?
            .iter()
            .filter_map(|k| {
                let v = obj.resolve_property_no_get_set(k.clone(), realm).ok()??; //TODO: This is absolutely not how this should be done (performance wise)

                let v = v.assert_value();

                if v.is_enumerable() {
                    Some(k.clone().into())
                } else {
                    None
                }
            })
            .collect();

        Ok(Array::with_elements(realm, keys)?.into_value())
    }

    #[prop("values")]
    fn values(obj: &Value, #[realm] realm: &mut Realm) -> ValueResult {
        let obj = match obj {
            Value::Object(obj) => obj,
            Value::Undefined | Value::Null => {
                return Err(Error::ty("Object.values() expects an object"))
            }
            _ => return Ok(Array::from_realm(realm)?.into_value()),
        };

        let keys = obj.enumerable_keys(realm)?;

        let mut props = Vec::with_capacity(keys.len());

        for key in keys {
            let Some(value) = obj.resolve_property_no_get_set(key, realm)? else {
                continue;
            };

            if !value.attributes().is_enumerable() {
                continue;
            }

            let value = match value {
                Property::Value(v, _) => v,
                Property::Getter(getter, _) => {
                    getter.call(Vec::new(), obj.clone().into(), realm)?
                }
            };

            props.push(value);
        }

        Ok(Array::with_elements(realm, props)?.into_value())
    }

    #[prop("setPrototypeOf")]
    fn set_prototype_of(
        obj: Value,
        proto: ObjectHandle,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        if obj.instance_of(&proto.clone().into(), realm)? {
            return Err(Error::ty("Cannot set prototype to itself"));
        }

        obj.as_object()?.set_prototype(proto.into(), realm)?;

        Ok(obj)
    }

    #[prop("preventExtensions")]
    fn prevent_extensions(o: &ObjectHandle) -> Res {
        o.prevent_extensions()
    }

    #[prop("isExtensible")]
    fn is_extensible(o: &ObjectHandle) -> bool {
        o.is_extensible()
    }

    #[prop("seal")]
    fn seal(o: &ObjectHandle) -> Res {
        o.seal()
    }

    #[prop("isSealed")]
    fn is_sealed(o: &ObjectHandle) -> bool {
        o.is_sealed()
    }

    #[prop("freeze")]
    fn freeze(o: &ObjectHandle) -> Res {
        o.freeze()
    }

    #[prop("isFrozen")]
    fn is_frozen(o: &ObjectHandle) -> bool {
        o.is_frozen()
    }
}
