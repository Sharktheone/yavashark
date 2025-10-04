#![allow(unused)]
use crate::array::Array;
use crate::value::{BoxedObj, DefinePropertyResult, Obj, Property};
use crate::{Error, InternalPropertyKey, NativeFunction, Object, ObjectHandle, ObjectOrNull, ObjectProperty, PropertyKey, Realm, Res, Value, Variable};
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


#[allow(unused)]
impl Obj for Proxy {
    fn define_property(&self, name: InternalPropertyKey, value: crate::value::Value, realm: &mut Realm) -> Res<DefinePropertyResult> {
        todo!()
    }

    fn define_property_attributes(&self, name: InternalPropertyKey, value: crate::value::Variable, realm: &mut Realm) -> Res<DefinePropertyResult> {
        todo!()
    }

    fn resolve_property(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<Option<Property>> {
        todo!()
    }

    fn get_own_property(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<Option<Property>> {
        todo!()
    }

    fn define_getter(&self, name: InternalPropertyKey, callback: ObjectHandle, realm: &mut Realm) -> Res {
        todo!()
    }

    fn define_setter(&self, name: InternalPropertyKey, callback: ObjectHandle, realm: &mut Realm) -> Res {
        todo!()
    }

    fn delete_property(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<Option<Property>> {
        todo!()
    }

    fn contains_own_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool> {
        todo!()
    }

    fn contains_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool> {
        todo!()
    }

    fn properties(&self, realm: &mut Realm) -> Res<Vec<(PropertyKey, crate::value::Value)>> {
        todo!()
    }

    fn keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>> {
        todo!()
    }

    fn values(&self, realm: &mut Realm) -> Res<Vec<crate::value::Value>> {
        todo!()
    }

    fn enumerable_properties(&self, realm: &mut Realm) -> Res<Vec<(PropertyKey, crate::value::Value)>> {
        todo!()
    }

    fn enumerable_keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>> {
        todo!()
    }

    fn enumerable_values(&self, realm: &mut Realm) -> Res<Vec<crate::value::Value>> {
        todo!()
    }

    fn clear_properties(&self, realm: &mut Realm) -> Res {
        todo!()
    }

    fn get_array_or_done(&self, idx: usize, realm: &mut Realm) -> Res<(bool, Option<crate::value::Value>)> {
        todo!()
    }

    fn prototype(&self, realm: &mut Realm) -> Res<ObjectOrNull> {
        todo!()
    }

    fn set_prototype(&self, prototype: ObjectOrNull, realm: &mut Realm) -> Res {
        todo!()
    }

    fn gc_refs(&self) -> Vec<GcRef<BoxedObj>> {
        todo!()
    }
}

// impl Obj for Proxy {
//     fn define_property(&self, name: Value, value: Value) -> Res {
//         if self.revoke.get() {
//             return self.inner.define_property(name, value);
//         }
//
//         Err(Error::new("not yet implemented"))
//     }
//
//     fn define_variable(&self, name: Value, value: Variable) -> Res {
//         if self.revoke.get() {
//             return self.inner.define_variable(name, value);
//         }
//
//         Err(Error::new("not yet implemented"))
//     }
//
//     fn resolve_property(&self, name: &Value) -> Result<Option<ObjectProperty>, Error> {
//         if self.revoke.get() {
//             return self.inner.deref().resolve_property(name);
//         }
//
//         Err(Error::new("not yet implemented"))
//     }
//
//     fn get_property(&self, name: &Value) -> Result<Option<ObjectProperty>, Error> {
//         if self.revoke.get() {
//             return self.inner.deref().get_property(name);
//         }
//
//         Err(Error::new("not yet implemented"))
//     }
//
//     fn define_getter(&self, name: Value, value: Value) -> Res {
//         if self.revoke.get() {
//             return self.inner.define_getter(name, value);
//         }
//
//         Err(Error::new("not yet implemented"))
//     }
//
//     fn define_setter(&self, name: Value, value: Value) -> Res {
//         if self.revoke.get() {
//             return self.inner.define_setter(name, value);
//         }
//
//         Err(Error::new("not yet implemented"))
//     }
//
//     fn delete_property(&self, name: &Value) -> Result<Option<Value>, Error> {
//         if self.revoke.get() {
//             return self.inner.delete_property(name);
//         }
//
//         Err(Error::new("not yet implemented"))
//     }
//
//     fn contains_key(&self, name: &Value) -> Result<bool, Error> {
//         if self.revoke.get() {
//             return self.inner.contains_key(name);
//         }
//
//         Err(Error::new("not yet implemented"))
//     }
//
//     fn has_key(&self, name: &Value) -> Result<bool, Error> {
//         if self.revoke.get() {
//             return self.inner.has_key(name);
//         }
//
//         Err(Error::new("not yet implemented"))
//     }
//
//     fn name(&self) -> String {
//         self.inner.name()
//     }
//
//     fn to_string(&self, realm: &mut Realm) -> Result<YSString, Error> {
//         self.inner.to_string(realm)
//     }
//
//     fn to_string_internal(&self) -> Result<YSString, Error> {
//         self.inner.to_string_internal()
//     }
//
//     fn properties(&self) -> Result<Vec<(Value, Value)>, Error> {
//         if self.revoke.get() {
//             return self.inner.properties();
//         }
//
//         Err(Error::new("not yet implemented"))
//     }
//
//     fn keys(&self) -> Result<Vec<Value>, Error> {
//         if self.revoke.get() {
//             return self.inner.keys();
//         }
//
//         Err(Error::new("not yet implemented"))
//     }
//
//     fn values(&self) -> Result<Vec<Value>, Error> {
//         if self.revoke.get() {
//             return self.inner.values();
//         }
//
//         Err(Error::new("not yet implemented"))
//     }
//
//     fn get_array_or_done(&self, index: usize) -> Result<(bool, Option<Value>), Error> {
//         if self.revoke.get() {
//             return self.inner.get_array_or_done(index);
//         }
//
//         Err(Error::new("not yet implemented"))
//     }
//
//     fn clear_values(&self) -> Res {
//         if self.revoke.get() {
//             return self.inner.clear_values();
//         }
//
//         Err(Error::new("not yet implemented"))
//     }
//
//     fn call(&self, realm: &mut Realm, args: Vec<Value>, this: Value) -> Result<Value, Error> {
//         if self.revoke.get() {
//             return self.inner.call(realm, args, this);
//         }
//
//         if let Some(apply) = self.handler.get_opt("apply", realm)? {
//             let apply = apply.to_object()?;
//
//             let arguments = Array::with_elements(realm, args)?;
//             apply.call(
//                 realm,
//                 vec![self.inner.clone().into(), this, arguments.into_value()],
//                 self.handler.clone().into(),
//             )
//         } else {
//             self.inner.call(realm, args, this)
//         }
//     }
//
//     fn is_function(&self) -> bool {
//         self.inner.is_function()
//     }
//
//     fn primitive(&self) -> Option<Value> {
//         self.inner.primitive()
//     }
//
//     fn prototype(&self) -> Result<ObjectProperty, Error> {
//         if self.revoke.get() {
//             return self.inner.prototype();
//         }
//
//         Err(Error::new("not yet implemented"))
//     }
//
//     fn set_prototype(&self, proto: ObjectProperty) -> Res {
//         if self.revoke.get() {
//             return self.inner.set_prototype(proto);
//         }
//
//         Err(Error::new("not yet implemented"))
//     }
//
//     fn constructor(&self) -> Result<ObjectProperty, Error> {
//         self.inner.constructor()
//     }
//
//     unsafe fn custom_gc_refs(&self) -> Vec<GcRef<BoxedObj>> {
//         vec![self.inner.get_ref(), self.handler.get_ref()]
//     }
//
//     fn class_name(&self) -> &'static str {
//         self.inner.class_name()
//     }
//
//     fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> Result<Value, Error> {
//         if self.revoke.get() {
//             return self.inner.construct(realm, args);
//         }
//
//         if let Some(construct) = self.handler.get_opt("construct", realm)? {
//             let construct = construct.to_object()?;
//             let arguments = Array::with_elements(realm, args)?;
//             construct.call(
//                 realm,
//                 vec![self.inner.clone().into(), arguments.into_value()],
//                 self.handler.clone().into(),
//             )
//         } else {
//             self.inner.construct(realm, args)
//         }
//     }
//
//     fn is_constructor(&self) -> bool {
//         self.inner.is_constructor()
//     }
//
//     unsafe fn inner_downcast(&self, ty: TypeId) -> Option<NonNull<()>> {
//         self.inner.inner_downcast(ty)
//     }
// }

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
