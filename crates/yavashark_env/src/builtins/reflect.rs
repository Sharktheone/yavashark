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
    pub fn apply(
        target: &Value,
        this: Value,
        args: Value,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let args = ArrayLike::new(args, realm)?.to_vec(realm)?;

        target.call(realm, args, this)
    }

    pub fn construct(
        target: &ObjectHandle,
        args: Value,
        new_target: &ObjectHandle,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        if !new_target.is_constructor() {
            return Err(Error::ty_error(format!(
                "{} is not a constructor",
                new_target.name()
            )));
        }

        let args = ArrayLike::new(args, realm)?.to_vec(realm)?;

        let val = target.construct(realm, args)?;

        if let Some(proto) = new_target.resolve_property(&"prototype".into(), realm)? {
            val.as_object()?
                .define_property("__proto__".into(), proto)?;
        }

        Ok(val)
    }

    #[prop("defineProperty")]
    #[must_use]
    pub fn define_property(target: ObjectHandle, prop: Value, desc: &ObjectHandle) -> bool {
        ObjectConstructor::define_property(target, prop, desc).is_ok()
    }

    #[prop("deleteProperty")]
    #[must_use]
    pub fn delete_property(target: &ObjectHandle, prop: &Value) -> bool {
        target.delete_property(prop).is_ok()
    }

    pub fn get(
        target: ObjectHandle,
        prop: &Value,
        receiver: Option<Value>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let prop = target.get_property(prop)?;

        let receiver = receiver.unwrap_or_else(|| target.into());

        prop.resolve(receiver, realm)
    }

    #[prop("getOwnPropertyDescriptor")]
    pub fn get_own_property_descriptor(
        target: &ObjectHandle,
        prop: &Value,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let obj = target.get();

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
