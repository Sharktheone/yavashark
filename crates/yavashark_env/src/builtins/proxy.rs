#![allow(unused)]
use crate::array::Array;
use crate::value::{
    Attributes, BoxedObj, DefinePropertyResult, IntoValue, Obj, Property, PropertyDescriptor,
    WeakObject,
};
use crate::{
    Error, InternalPropertyKey, NativeFunction, Object, ObjectHandle, ObjectOrNull, ObjectProperty,
    PrimitiveValue, PropertyKey, Realm, Res, Value, Variable, WeakObjectHandle,
};
use std::any::TypeId;
use std::cell::{Cell, RefCell};
use std::ops::Deref;
use std::ptr::NonNull;
use swc_ecma_ast::Prop;
use yavashark_garbage::GcRef;
use yavashark_macro::props;
use yavashark_string::YSString;

#[derive(Debug)]
pub struct Proxy {
    inner: ObjectHandle,
    handler: ObjectHandle,
    this: RefCell<Option<ObjectHandle>>,
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

        if let Some(define_property) = self.handler.get_opt("set", realm)? {
            let define_property = define_property.to_object()?;

            let result = define_property.call(
                vec![self.inner.clone().into(), name.into(), value, self.this()],
                self.handler.clone().into(),
                realm,
            )?;

            if result.is_truthy() {
                Ok(DefinePropertyResult::Handled)
            } else {
                Err(Error::ty(
                    "Proxy handler's defineProperty method returned false",
                ))
            }
        } else {
            self.inner.define_property(name, value, realm)
        }
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

        if let Some(define_property) = self.handler.get_opt("set", realm)? {
            let define_property = define_property.to_object()?;

            let result = define_property.call(
                vec![
                    self.inner.clone().into(),
                    name.into(),
                    value.value,
                    self.this(),
                ],
                self.handler.clone().into(),
                realm,
            )?;

            if result.is_truthy() {
                Ok(DefinePropertyResult::Handled)
            } else {
                Err(Error::ty(
                    "Proxy handler's defineProperty method returned false",
                ))
            }
        } else {
            self.inner.define_property_attributes(name, value, realm)
        }
    }

    fn resolve_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Result<Option<Property>, Error> {
        if self.revoke.get() {
            return self.inner.deref().resolve_property(name, realm);
        }

        if let Some(get) = self.handler.get_opt("get", realm)? {
            let get = get.to_object()?;
            let result = get.call(
                vec![
                    self.inner.clone().into(),
                    name.into(),
                    self.handler.clone().into(),
                    self.this(),
                ],
                self.handler.clone().into(),
                realm,
            )?;
            Ok(Some(Property::Value(result, Attributes::new())))
        } else {
            self.inner.deref().resolve_property(name, realm)
        }
    }

    fn get_own_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Result<Option<Property>, Error> {
        if self.revoke.get() {
            return self.inner.deref().get_own_property(name, realm);
        }

        if let Some(get_own_property) = self.handler.get_opt("get", realm)? {
            let get_own_property = get_own_property.to_object()?;
            let result = get_own_property.call(
                vec![self.inner.clone().into(), name.into(), self.this()],
                self.handler.clone().into(),
                realm,
            )?;
            if result.is_undefined() {
                Ok(None)
            } else {
                let desc: PropertyDescriptor = result.into();
                Ok(Some(desc.into()))
            }
        } else {
            self.inner.deref().get_own_property(name, realm)
        }
    }

    fn define_getter(
        &self,
        name: InternalPropertyKey,
        value: ObjectHandle,
        realm: &mut Realm,
    ) -> Res {
        self.inner.define_getter(name, value, realm)
    }

    fn define_getter_attributes(
        &self,
        name: InternalPropertyKey,
        callback: ObjectHandle,
        attributes: Attributes,
        realm: &mut Realm,
    ) -> Res {
        self.inner
            .define_getter_attributes(name, callback, attributes, realm)
    }

    fn define_setter(
        &self,
        name: InternalPropertyKey,
        value: ObjectHandle,
        realm: &mut Realm,
    ) -> Res {
        self.inner.define_setter(name, value, realm)
    }

    fn define_setter_attributes(
        &self,
        name: InternalPropertyKey,
        callback: ObjectHandle,
        attributes: Attributes,
        realm: &mut Realm,
    ) -> Res {
        self.inner
            .define_setter_attributes(name, callback, attributes, realm)
    }

    fn define_empty_accessor(
        &self,
        name: InternalPropertyKey,
        attributes: Attributes,
        realm: &mut Realm,
    ) -> Res {
        self.inner.define_empty_accessor(name, attributes, realm)
    }

    fn delete_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        if self.revoke.get() {
            return self.inner.delete_property(name, realm);
        }

        if let Some(delete_property) = self.handler.get_opt("deleteProperty", realm)? {
            let delete_property = delete_property.to_object()?;
            let result = delete_property.call(
                vec![self.inner.clone().into(), name.into(), self.this()],
                self.handler.clone().into(),
                realm,
            )?;

            if result.is_truthy() {
                Ok(Some(Property::Value(Value::Undefined, Attributes::new())))
            } else {
                Err(Error::ty(
                    "Proxy handler's deleteProperty method returned false",
                ))
            }
        } else {
            self.inner.delete_property(name, realm)
        }
    }

    fn contains_own_key(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Result<bool, Error> {
        if self.revoke.get() {
            return self.inner.contains_own_key(name, realm);
        }

        if let Some(has) = self.handler.get_opt("has", realm)? {
            let has = has.to_object()?;
            let result = has.call(
                vec![self.inner.clone().into(), name.into(), self.this()],
                self.handler.clone().into(),
                realm,
            )?;

            Ok(result.is_truthy())
        } else {
            self.inner.contains_own_key(name, realm)
        }
    }

    fn contains_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Result<bool, Error> {
        if self.revoke.get() {
            return self.inner.contains_key(name, realm);
        }

        if let Some(has) = self.handler.get_opt("has", realm)? {
            let has = has.to_object()?;
            let result = has.call(
                vec![self.inner.clone().into(), name.into(), self.this()],
                self.handler.clone().into(),
                realm,
            )?;

            Ok(result.is_truthy())
        } else {
            self.inner.contains_key(name, realm)
        }
    }

    // fn to_string(&self, realm: &mut Realm) -> Result<YSString, Error> {
    //     self.inner.to_string(realm)
    // }
    //
    // fn to_string_internal(&self) -> Result<YSString, Error> {
    //     self.inner.to_string_internal()
    // }

    fn properties(&self, realm: &mut Realm) -> Res<Vec<(PropertyKey, Property)>> {
        Ok(self
            .inner
            .properties(realm)?
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect())
    }

    fn keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>> {
        self.inner.keys(realm)
    }

    fn values(&self, realm: &mut Realm) -> Res<Vec<Property>> {
        Ok(self
            .inner
            .values(realm)?
            .into_iter()
            .map(|v| v.into())
            .collect())
    }

    fn enumerable_properties(&self, realm: &mut Realm) -> Res<Vec<(PropertyKey, Property)>> {
        self.inner.enumerable_properties(realm)
    }

    fn enumerable_keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>> {
        self.inner.enumerable_keys(realm)
    }

    fn enumerable_values(&self, realm: &mut Realm) -> Res<Vec<Property>> {
        self.inner.enumerable_values(realm)
    }

    fn clear_properties(&self, realm: &mut Realm) -> Res {
        self.inner.clear_properties(realm)
    }

    fn get_array_or_done(
        &self,
        index: usize,
        realm: &mut Realm,
    ) -> Result<(bool, Option<Value>), Error> {
        if self.revoke.get() {
            return self.inner.get_array_or_done(index, realm);
        }

        if let Some(get) = self.handler.get_opt("get", realm)? {
            let get = get.to_object()?;
            let result = get.call(
                vec![
                    self.inner.clone().into(),
                    (index as u32).into(),
                    self.handler.clone().into(),
                    self.this(),
                ],
                self.handler.clone().into(),
                realm,
            )?;
            Ok((true, Some(result)))
        } else {
            self.inner.get_array_or_done(index, realm)
        }
    }

    fn call(&self, args: Vec<Value>, this: Value, realm: &mut Realm) -> Result<Value, Error> {
        if self.revoke.get() {
            return self.inner.call(args, this, realm);
        }

        if let Some(apply) = self.handler.get_opt("apply", realm)? {
            let apply = apply.to_object()?;

            let arguments = Array::with_elements(realm, args)?;
            apply.call(
                vec![
                    self.inner.clone().into(),
                    this,
                    arguments.into_value(),
                    self.this(),
                ],
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

        if let Some(get_prototype) = self.handler.get_opt("getPrototypeOf", realm)? {
            let get_prototype = get_prototype.to_object()?;
            let result = get_prototype.call(
                vec![self.inner.clone().into(), self.this()],
                self.handler.clone().into(),
                realm,
            )?;

            if result.is_null() || result.is_object() {
                result.try_into()
            } else {
                Err(Error::ty(
                    "Proxy handler's getPrototypeOf method did not return an object or null",
                ))
            }
        } else {
            self.inner.prototype(realm)
        }
    }

    fn set_prototype(&self, proto: ObjectOrNull, realm: &mut Realm) -> Res {
        if self.revoke.get() {
            return self.inner.set_prototype(proto, realm);
        }

        if let Some(set_prototype) = self.handler.get_opt("setPrototypeOf", realm)? {
            let set_prototype = set_prototype.to_object()?;
            let result = set_prototype.call(
                vec![self.inner.clone().into(), proto.into(), self.this()],
                self.handler.clone().into(),
                realm,
            )?;

            if result.is_truthy() {
                Ok(())
            } else {
                Err(Error::ty(
                    "Proxy handler's setPrototypeOf method returned false",
                ))
            }
        } else {
            self.inner.set_prototype(proto, realm)
        }
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
                    vec![
                        self.inner.clone().into(),
                        arguments.into_value(),
                        self.this(),
                    ],
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
        if ty == TypeId::of::<Self>() {
            return Some(NonNull::from(self).cast());
        }

        self.inner.inner_downcast(ty)
    }

    fn gc_refs(&self) -> Vec<GcRef<BoxedObj>> {
        vec![self.inner.get_ref(), self.handler.get_ref()]
    }
}

#[props(intrinsic_name = proxy)]
impl Proxy {
    #[constructor]
    pub fn construct(target: ObjectHandle, handler: ObjectHandle) -> Res<ObjectHandle> {
        let this = Self {
            inner: target,
            handler,
            revoke: Cell::new(false),
            this: RefCell::new(None),
        }
        .into_object();

        if let Some(proxy) = this.downcast::<Self>() {
            proxy.this.replace(Some(this.clone()));
        } else {
            return Err(Error::ty("Failed to create proxy"));
        }

        Ok(this)
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
            this: RefCell::new(None),
        }
        .into_object();

        if let Some(p) = proxy.downcast::<Self>() {
            p.this.replace(Some(proxy.clone()));
        } else {
            return Err(Error::ty("Failed to create proxy"));
        }

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

    fn this(&self) -> Value {
        self.this
            .borrow()
            .as_ref()
            .cloned()
            .map_or(Value::Undefined, Into::into)
        // .map_or(Value::Undefined, |w| w.upgrade().map_or(Value::Undefined, Into::into))
    }
}
