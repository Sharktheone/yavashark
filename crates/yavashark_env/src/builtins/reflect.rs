use crate::constructor::ObjectConstructor;
use crate::partial_init::Initializer;
use crate::utils::ArrayLike;
use crate::value::{Obj, Property, PropertyDescriptor};
use crate::{
    Error, InternalPropertyKey, MutObject, ObjectHandle, ObjectOrNull, Realm, Res, Value,
    ValueResult,
};
use std::cell::RefCell;
use yavashark_macro::{object, properties_new};

#[object]
#[derive(Debug)]
pub struct Reflect {}

impl Reflect {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(realm: &mut Realm) -> Res<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableReflect {
                object: MutObject::with_proto(realm.intrinsics.obj.clone()),
            }),
        };

        this.initialize(realm)?;

        Ok(this.into_object())
    }
}

#[properties_new(raw)]
impl Reflect {
    //28.1.1 Reflect.apply ( target, thisArgument, argumentsList ), https://tc39.es/ecma262/#sec-reflection
    pub fn apply(
        target: &Value,
        this: Value,
        args: Value,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        //This function performs the following steps when called:
        //1. If IsCallable(target) is false, throw a TypeError exception. - done by `target.call()`
        //2. Let args be ? CreateListFromArrayLike(argumentsList).
        let args = ArrayLike::new(args, realm)?.to_vec(realm)?;
        //3. Perform PrepareForTailCall(). TODO

        //4. Return ? Call(target, thisArgument, args).
        target.call(realm, args, this)
    }

    //28.1.2 Reflect.construct ( target, argumentsList [ , newTarget ] ), https://tc39.es/ecma262/#sec-reflection
    pub fn construct(
        target: &ObjectHandle,
        args: Value,
        new_target: &Option<ObjectHandle>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        //This function performs the following steps when called:

        //     1. If IsConstructor(target) is false, throw a TypeError exception.
        if !target.is_constructable() {
            return Err(Error::ty_error(format!(
                "{} is not a constructor",
                target.name()
            )));
        }
        //     2. If newTarget is not present, set newTarget to target.
        let proto = if let Some(new_target) = new_target {
            //     3. Else if IsConstructor(newTarget) is false, throw a TypeError exception.
            if !new_target.is_constructable() {
                return Err(Error::ty_error(format!(
                    "{} is not a constructor",
                    new_target.name()
                )));
            }
            let proto = new_target.get("prototype", realm)?;

            if proto.is_object() {
                proto
            } else {
                target.get("prototype", realm)?
            }
        } else {
            target.get("prototype", realm)?
        };

        //     4. Let args be ? CreateListFromArrayLike(argumentsList).
        let args = ArrayLike::new(args, realm)?.to_vec(realm)?;

        //     5. Return ? Construct(target, args, newTarget).
        let val = target.construct(args, realm)?;

        let proto = if proto.is_null() {
            ObjectOrNull::Null
        } else {
            ObjectOrNull::Object(proto.to_object()?)
        };

        val.set_prototype(proto, realm)?;

        Ok(val)
    }

    //28.1.3 Reflect.defineProperty ( target, propertyKey, attributes ), https://tc39.es/ecma262/#sec-reflection
    #[prop("defineProperty")]
    #[must_use]
    pub fn define_property(
        target: ObjectHandle,
        prop: InternalPropertyKey,
        desc: PropertyDescriptor,
        #[realm] realm: &mut Realm,
    ) -> bool {
        //This function performs the following steps when called:
        //1. If target is not an Object, throw a TypeError exception. - done by the caller
        //2. Let key be ? ToPropertyKey(propertyKey). TODO
        //3. Let desc be ? ToPropertyDescriptor(attributes).
        //4. Return ? target.[[DefineOwnProperty]](key, desc).
        target.define_descriptor(prop, desc, realm).is_ok()
    }

    //28.1.4 Reflect.deleteProperty ( target, propertyKey ), https://tc39.es/ecma262/#sec-reflection
    #[prop("deleteProperty")]
    pub fn delete_property(
        target: &ObjectHandle,
        prop: InternalPropertyKey,
        #[realm] realm: &mut Realm,
    ) -> Res<bool> {
        //This function performs the following steps when called:
        //1. If target is not an Object, throw a TypeError exception. - done by the caller
        //2. Let key be ? ToPropertyKey(propertyKey). TODO
        //3. Return ? target.[[Delete]](key).
        Ok(target.delete_property(prop, realm)?.is_some())
    }

    //28.1.5 Reflect.get ( target, propertyKey [ , receiver ] ), https://tc39.es/ecma262/#sec-reflection
    pub fn get(
        target: ObjectHandle,
        prop: &Value,
        receiver: Option<Value>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        //This function performs the following steps when called:
        //1. If target is not an Object, throw a TypeError exception. - done by the caller
        //2. Let key be ? ToPropertyKey(propertyKey). TODO
        match target.resolve_property_no_get_set(prop, realm)? {
            Some(Property::Value(v, _)) => Ok(v),
            Some(Property::Getter(getter, _)) => {
                let recv = receiver.unwrap_or_else(|| target.clone().into());
                getter.call(Vec::new(), recv, realm)
            }
            None => Ok(Value::Undefined),
        }

        // Ok(prop.unwrap_or(Value::Undefined))
    }

    #[prop("getOwnPropertyDescriptor")]
    pub fn get_own_property_descriptor(
        target: &ObjectHandle,
        prop: InternalPropertyKey,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let Some(prop) = target.get_property_descriptor(prop, realm)? else {
            return Ok(Value::Undefined);
        };

        prop.into_value(realm)
    }

    #[prop("getPrototypeOf")]
    pub fn get_prototype_of(target: ObjectHandle, #[realm] realm: &mut Realm) -> ValueResult {
        Ok(target.prototype(realm)?.into())
    }

    pub fn has(
        target: &ObjectHandle,
        prop: InternalPropertyKey,
        #[realm] realm: &mut Realm,
    ) -> Res<bool> {
        target.contains_own_key(prop, realm)
    }

    #[prop("isExtensible")]
    #[must_use]
    pub const fn is_extensible(_target: &ObjectHandle) -> bool {
        true
    }

    #[prop("ownKeys")]
    pub fn own_keys(target: ObjectHandle, #[realm] realm: &mut Realm) -> ValueResult {
        ObjectConstructor::keys_js(&target.into(), realm)
    }

    #[prop("preventExtensions")]
    #[must_use]
    pub const fn prevent_extensions(_target: &ObjectHandle) -> bool {
        false
    }

    #[must_use]
    pub fn set(
        target: &ObjectHandle,
        prop: InternalPropertyKey,
        value: Value,
        _receiver: Option<Value>,
        #[realm] realm: &mut Realm,
    ) -> bool {
        target.define_property(prop, value, realm).is_ok()
    }

    #[prop("setPrototypeOf")]
    pub fn set_prototype_of(
        target: &ObjectHandle,
        proto: Value,
        #[realm] realm: &mut Realm,
    ) -> Res<bool> {
        let proto = if proto.is_null() {
            ObjectOrNull::Null
        } else {
            ObjectOrNull::Object(proto.to_object()?)
        };

        Ok(target.set_prototype(proto, realm).is_ok())
    }
}

impl Initializer<ObjectHandle> for Reflect {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
        Reflect::new(realm)
    }
}
