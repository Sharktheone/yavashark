use crate::constructor::ObjectConstructor;
use crate::utils::ArrayLike;
use crate::{Error, MutObject, Object, ObjectHandle, Realm, Res, Value, ValueResult};
use std::cell::RefCell;
use yavashark_macro::{object, properties_new};
use yavashark_value::Obj;

#[object]
#[derive(Debug)]
pub struct Reflect {}

impl Reflect {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(proto: Value, func_proto: Value) -> Res<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableReflect {
                object: MutObject::with_proto(proto),
            }),
        };

        this.initialize(func_proto)?;

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
    ) -> ValueResult {
        //This function performs the following steps when called:

        //     1. If IsConstructor(target) is false, throw a TypeError exception.
        if !target.is_constructor() {
            return Err(Error::ty_error(format!(
                "{} is not a constructor",
                target.name()
            )));
        }
        //     2. If newTarget is not present, set newTarget to target.
        let proto = if let Some(new_target) = new_target {
            //     3. Else if IsConstructor(newTarget) is false, throw a TypeError exception.
            if !new_target.is_constructor() {
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
        let val = target.construct(realm, args)?;

        val.as_object()?.set_prototype(proto.into())?;

        Ok(val)
    }

    //28.1.3 Reflect.defineProperty ( target, propertyKey, attributes ), https://tc39.es/ecma262/#sec-reflection
    #[prop("defineProperty")]
    #[must_use]
    pub fn define_property(target: ObjectHandle, prop: &Value, desc: &ObjectHandle) -> bool {
        //This function performs the following steps when called:
        //1. If target is not an Object, throw a TypeError exception. - done by the caller
        //2. Let key be ? ToPropertyKey(propertyKey). TODO
        //3. Let desc be ? ToPropertyDescriptor(attributes).
        //4. Return ? target.[[DefineOwnProperty]](key, desc).
        ObjectConstructor::define_property(target, prop, desc).is_ok()
    }

    //28.1.4 Reflect.deleteProperty ( target, propertyKey ), https://tc39.es/ecma262/#sec-reflection
    #[prop("deleteProperty")]
    #[must_use]
    pub fn delete_property(target: &ObjectHandle, prop: &Value) -> bool {
        //This function performs the following steps when called:
        //1. If target is not an Object, throw a TypeError exception. - done by the caller
        //2. Let key be ? ToPropertyKey(propertyKey). TODO
        //3. Return ? target.[[Delete]](key).
        target.delete_property(prop).is_ok()
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
        let prop = target.get_property(prop)?;

        //3. If receiver is not present, then
        //    a. Set receiver to target.
        let receiver = receiver.unwrap_or_else(|| target.into());

        //4. Return ? target.[[Get]](key, receiver).
        prop.resolve(receiver, realm)
    }

    #[prop("getOwnPropertyDescriptor")]
    pub fn get_own_property_descriptor(
        target: &ObjectHandle,
        prop: &Value,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let obj = target.guard();

        let Some(prop) = obj.get_property(prop)? else {
            return Ok(Value::Undefined);
        };

        let desc = Object::new(realm);

        prop.descriptor(&desc)?;

        Ok(desc.into())
    }

    #[prop("getPrototypeOf")]
    pub fn get_prototype_of(target: ObjectHandle, #[realm] realm: &mut Realm) -> ValueResult {
        target.prototype()?.resolve(target.into(), realm)
    }

    pub fn has(target: &ObjectHandle, prop: &Value) -> Res<bool> {
        target.contains_key(prop)
    }

    #[prop("isExtensible")]
    #[must_use]
    pub const fn is_extensible(_target: &ObjectHandle) -> bool {
        true
    }

    #[prop("ownKeys")]
    pub fn own_keys(target: &ObjectHandle, #[realm] realm: &Realm) -> ValueResult {
        ObjectConstructor::keys_js(target, realm)
    }

    #[prop("preventExtensions")]
    #[must_use]
    pub const fn prevent_extensions(_target: &ObjectHandle) -> bool {
        false
    }

    #[must_use]
    pub fn set(target: &ObjectHandle, prop: Value, value: Value, _receiver: Option<Value>) -> bool {
        target.define_property(prop, value).is_ok()
    }

    #[prop("setPrototypeOf")]
    #[must_use]
    pub fn set_prototype_of(target: &ObjectHandle, proto: Value) -> bool {
        target.set_prototype(proto.into()).is_ok()
    }
}
