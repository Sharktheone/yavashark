#![allow(unused)]
use std::any::TypeId;
use std::ptr::NonNull;
use yavashark_garbage::GcRef;
use yavashark_macro::props;
use yavashark_string::YSString;
use yavashark_value::{BoxedObj, Obj};
use crate::{Error, ObjectHandle, ObjectProperty, Realm, Res, Value, Variable};
use crate::array::Array;

#[derive(Debug)]
pub struct Proxy {
    inner: ObjectHandle,
    handler: ObjectHandle,
}


impl Obj<Realm> for Proxy {
    fn define_property(&self, name: Value, value: Value) -> Res {
        Err(Error::new("not yet implemented"))
    }

    fn define_variable(&self, name: Value, value: Variable) -> Res {
        Err(Error::new("not yet implemented"))
    }

    fn resolve_property(&self, name: &Value) -> Result<Option<ObjectProperty>, Error> {
        Err(Error::new("not yet implemented"))
    }

    fn get_property(&self, name: &Value) -> Result<Option<ObjectProperty>, Error> {
        Err(Error::new("not yet implemented"))
    }

    fn define_getter(&self, name: Value, value: Value) -> Res {
        Err(Error::new("not yet implemented"))
    }

    fn define_setter(&self, name: Value, value: Value) -> Res {
        Err(Error::new("not yet implemented"))
    }

    fn delete_property(&self, name: &Value) -> Result<Option<Value>, Error> {
        Err(Error::new("not yet implemented"))
    }

    fn contains_key(&self, name: &Value) -> Result<bool, Error> {
        Err(Error::new("not yet implemented"))
    }

    fn has_key(&self, name: &Value) -> Result<bool, Error> {
        Err(Error::new("not yet implemented"))
    }

    fn name(&self) -> String {
        self.inner.name()
    }

    fn to_string(&self, realm: &mut Realm) -> Result<YSString, Error> {
        self.inner.to_string(realm)
    }

    fn to_string_internal(&self) -> Result<YSString, Error> {
        self.inner.to_string_internal()
    }

    fn properties(&self) -> Result<Vec<(Value, Value)>, Error> {
        Err(Error::new("not yet implemented"))
    }

    fn keys(&self) -> Result<Vec<Value>, Error> {
        Err(Error::new("not yet implemented"))
    }

    fn values(&self) -> Result<Vec<Value>, Error> {
        Err(Error::new("not yet implemented"))
    }

    fn get_array_or_done(&self, index: usize) -> Result<(bool, Option<Value>), Error> {
        Err(Error::new("not yet implemented"))
    }

    fn clear_values(&self) -> Res {
        Err(Error::new("not yet implemented"))
    }

    fn call(&self, realm: &mut Realm, args: Vec<Value>, this: Value) -> Result<Value, Error> {
        if let Some(apply) = self.handler.get_opt("apply", realm)? {
            let apply = apply.to_object()?;
            
            let arguments = Array::with_elements(realm, args)?;
            apply.call(
                realm,
                vec![self.inner.clone().into(), this, arguments.into_value()],
                self.handler.clone().into(),
            )
            
            
        } else {
            self.inner.call(realm, args, this)
        }
    }

    fn is_function(&self) -> bool {
        self.inner.is_function()
    }

    fn primitive(&self) -> Option<Value> {
        self.inner.primitive()
    }

    fn prototype(&self) -> Result<ObjectProperty, Error> {
        Err(Error::new("not yet implemented"))
    }

    fn set_prototype(&self, proto: ObjectProperty) -> Res {
        Err(Error::new("not yet implemented"))
    }

    fn constructor(&self) -> Result<ObjectProperty, Error> {
        self.inner.constructor()
    }

    unsafe fn custom_gc_refs(&self) -> Vec<GcRef<BoxedObj<Realm>>> {
        vec![
            self.inner.get_ref(),
            self.handler.get_ref(),
        ]
    }

    fn class_name(&self) -> &'static str {
        self.inner.class_name()
    }

    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> Result<Value, Error> {
        if let Some(construct) = self.handler.get_opt("construct", realm)? {
            let construct = construct.to_object()?;
            let arguments = Array::with_elements(realm, args)?;
            construct.call(
                realm,
                vec![self.inner.clone().into(), arguments.into_value()],
                self.handler.clone().into(),
            )
        } else {
            self.inner.construct(realm, args)
        }
    }

    fn is_constructor(&self) -> bool {
        self.inner.is_constructor()
    }

    unsafe fn inner_downcast(&self, ty: TypeId) -> Option<NonNull<()>> {
        self.inner.inner_downcast(ty)
    }
}

#[props]
impl Proxy {
    #[constructor]
    pub fn construct(target: ObjectHandle, handler: ObjectHandle) -> ObjectHandle {
        Self {
            inner: target,
            handler,
        }
            .into_object()
    }
    
    
}