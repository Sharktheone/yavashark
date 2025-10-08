#![allow(unused)]
use crate::array::Array;
use crate::value::{BoxedObj, DefinePropertyResult, Obj, Property};
use crate::{
    Error, InternalPropertyKey, NativeFunction, Object, ObjectHandle, ObjectOrNull, ObjectProperty,
    PrimitiveValue, PropertyKey, Realm, Res, Value, Variable,
};
use std::any::TypeId;
use std::cell::Cell;
use std::ops::Deref;
use std::ptr::NonNull;
use yavashark_garbage::GcRef;
use yavashark_macro::props;
use yavashark_string::YSString;

#[derive(Debug)]
pub struct Proxy {
    inner: ObjectHandle,
    handler: ObjectHandle,
    revoke: Cell<bool>,
}

impl Obj for Proxy {
    fn define_property(
        &self,
        name: InternalPropertyKey,
        value: Value,
        realm: &mut Realm,
    ) -> Res<DefinePropertyResult> {
        if self.revoke.get() {
            return self.inner.define_property(name, value, realm);
        }

        Err(Error::new("not yet implemented"))
    }

    fn define_property_attributes(
        &self,
        name: InternalPropertyKey,
        value: Variable,
        realm: &mut Realm,
    ) -> Res<DefinePropertyResult> {
        if self.revoke.get() {
            return self.inner.define_property_attributes(name, value, realm);
        }

        Err(Error::new("not yet implemented"))
    }

    fn resolve_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Result<Option<Property>, Error> {
        if self.revoke.get() {
            return self.inner.deref().resolve_property(name, realm);
        }

        Err(Error::new("not yet implemented"))
    }

    fn get_own_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Result<Option<Property>, Error> {
        if self.revoke.get() {
            return self.inner.deref().get_own_property(name, realm);
        }

        Err(Error::new("not yet implemented"))
    }

    fn define_getter(
        &self,
        name: InternalPropertyKey,
        value: ObjectHandle,
        realm: &mut Realm,
    ) -> Res {
        if self.revoke.get() {
            return self.inner.define_getter(name, value, realm);
        }

        Err(Error::new("not yet implemented"))
    }

    fn define_setter(
        &self,
        name: InternalPropertyKey,
        value: ObjectHandle,
        realm: &mut Realm,
    ) -> Res {
        if self.revoke.get() {
            return self.inner.define_setter(name, value, realm);
        }

        Err(Error::new("not yet implemented"))
    }

    fn delete_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        if self.revoke.get() {
            return self.inner.delete_property(name, realm);
        }

        Err(Error::new("not yet implemented"))
    }

    fn contains_own_key(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Result<bool, Error> {
        if self.revoke.get() {
            return self.inner.contains_own_key(name, realm);
        }

        Err(Error::new("not yet implemented"))
    }

    fn contains_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Result<bool, Error> {
        if self.revoke.get() {
            return self.inner.contains_key(name, realm);
        }

        Err(Error::new("not yet implemented"))
    }

    // fn to_string(&self, realm: &mut Realm) -> Result<YSString, Error> {
    //     self.inner.to_string(realm)
    // }
    //
    // fn to_string_internal(&self) -> Result<YSString, Error> {
    //     self.inner.to_string_internal()
    // }

    fn properties(&self, realm: &mut Realm) -> Res<Vec<(PropertyKey, Value)>> {
        if self.revoke.get() {
            return self.inner.properties(realm);
        }

        Err(Error::new("not yet implemented"))
    }

    fn keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>> {
        if self.revoke.get() {
            return self.inner.keys(realm);
        }

        Err(Error::new("not yet implemented"))
    }

    fn values(&self, realm: &mut Realm) -> Res<Vec<Value>> {
        if self.revoke.get() {
            return self.inner.values(realm);
        }

        Err(Error::new("not yet implemented"))
    }

    fn enumerable_properties(
        &self,
        realm: &mut Realm,
    ) -> Res<Vec<(PropertyKey, crate::value::Value)>> {
        if self.revoke.get() {
            return self.inner.enumerable_properties(realm);
        }

        Err(Error::new("not yet implemented"))
    }

    fn enumerable_keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>> {
        if self.revoke.get() {
            return self.inner.enumerable_keys(realm);
        }

        Err(Error::new("not yet implemented"))
    }

    fn enumerable_values(&self, realm: &mut Realm) -> Res<Vec<crate::value::Value>> {
        if self.revoke.get() {
            return self.inner.enumerable_values(realm);
        }

        Err(Error::new("not yet implemented"))
    }

    fn clear_properties(&self, realm: &mut Realm) -> Res {
        if self.revoke.get() {
            return self.inner.clear_properties(realm);
        }

        Err(Error::new("not yet implemented"))
    }

    fn get_array_or_done(
        &self,
        index: usize,
        realm: &mut Realm,
    ) -> Result<(bool, Option<Value>), Error> {
        if self.revoke.get() {
            return self.inner.get_array_or_done(index, realm);
        }

        Err(Error::new("not yet implemented"))
    }

    fn call(&self, args: Vec<Value>, this: Value, realm: &mut Realm) -> Result<Value, Error> {
        if self.revoke.get() {
            return self.inner.call(args, this, realm);
        }

        if let Some(apply) = self.handler.get_opt("apply", realm)? {
            let apply = apply.to_object()?;

            let arguments = Array::with_elements(realm, args)?;
            apply.call(
                vec![self.inner.clone().into(), this, arguments.into_value()],
                self.handler.clone().into(),
                realm,
            )
        } else {
            self.inner.call(args, this, realm)
        }
    }

    fn is_callable(&self) -> bool {
        self.inner.is_callable()
    }

    fn primitive(&self, realm: &mut Realm) -> Res<Option<PrimitiveValue>> {
        self.inner.primitive(realm)
    }

    fn prototype(&self, realm: &mut Realm) -> Res<ObjectOrNull> {
        if self.revoke.get() {
            return self.inner.prototype(realm);
        }

        Err(Error::new("not yet implemented"))
    }

    fn set_prototype(&self, proto: ObjectOrNull, realm: &mut Realm) -> Res {
        if self.revoke.get() {
            return self.inner.set_prototype(proto, realm);
        }

        Err(Error::new("not yet implemented"))
    }

    fn construct(&self, args: Vec<Value>, realm: &mut Realm) -> Result<ObjectHandle, Error> {
        if self.revoke.get() {
            return self.inner.construct(args, realm);
        }

        if let Some(construct) = self.handler.get_opt("construct", realm)? {
            let construct = construct.to_object()?;
            let arguments = Array::with_elements(realm, args)?;
            construct
                .call(
                    vec![self.inner.clone().into(), arguments.into_value()],
                    self.handler.clone().into(),
                    realm,
                )?
                .to_object()
        } else {
            self.inner.construct(args, realm)
        }
    }

    fn is_constructable(&self) -> bool {
        self.inner.is_constructable()
    }

    fn name(&self) -> String {
        self.inner.name()
    }

    fn class_name(&self) -> &'static str {
        self.inner.class_name()
    }

    unsafe fn inner_downcast(&self, ty: TypeId) -> Option<NonNull<()>> {
        self.inner.inner_downcast(ty)
    }

    fn gc_refs(&self) -> Vec<GcRef<BoxedObj>> {
        self.inner.gc_refs() //TODO: this is not correct
    }
}

#[props]
impl Proxy {
    #[constructor]
    #[must_use]
    pub fn construct(target: ObjectHandle, handler: ObjectHandle) -> ObjectHandle {
        Self {
            inner: target,
            handler,
            revoke: Cell::new(false),
        }
        .into_object()
    }

    pub fn revocable(
        target: ObjectHandle,
        handler: ObjectHandle,
        realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let proxy = Self {
            inner: target,
            handler,
            revoke: Cell::new(false),
        }
        .into_object();

        let mut p = proxy.clone();

        let revoke = NativeFunction::new(
            "revoke",
            move |_, _, _| {
                if let Some(proxy) = p.downcast::<Self>() {
                    proxy.revoke.set(true);
                }

                Ok(Value::Undefined)
            },
            realm,
        );

        let ret = Object::new(realm);

        ret.set("proxy", proxy, realm)?;
        ret.set("revoke", revoke, realm)?;

        Ok(ret)
    }
}
