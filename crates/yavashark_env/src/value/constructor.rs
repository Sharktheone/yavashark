use crate::error::Error;
use crate::value::{BoxedObj, Obj, Value};
use crate::{ObjectHandle, Realm};
use std::fmt::Debug;
use yavashark_garbage::GcRef;

pub trait Constructor: Debug + Obj {
    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> Result<ObjectHandle, Error>;

    fn is_constructable(&self) -> bool {
        true
    }
}

pub trait ConstructorFn: Debug {
    fn gc_untyped_ref(&self) -> Option<GcRef<BoxedObj>>;
    fn construct(&self, args: Vec<Value>, this: Value, realm: &mut Realm) -> Result<(), Error>;
}

pub trait InstanceFieldInitializer: Debug {
    fn gc_untyped_ref(&self) -> Option<GcRef<BoxedObj>>;
    fn initialize(&self, this: Value, realm: &mut Realm) -> Result<(), Error>;
}

#[derive(Debug)]
pub struct NoOpConstructorFn;

impl ConstructorFn for NoOpConstructorFn {
    fn gc_untyped_ref(&self) -> Option<GcRef<BoxedObj>> {
        None
    }

    fn construct(&self, _args: Vec<Value>, _this: Value, _realm: &mut Realm) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct ConstantFieldInitializer {
    pub key: crate::InternalPropertyKey,
    pub value: Value,
    pub is_private: bool,
}

impl InstanceFieldInitializer for ConstantFieldInitializer {
    fn gc_untyped_ref(&self) -> Option<GcRef<BoxedObj>> {
        None
    }

    fn initialize(&self, this: Value, realm: &mut Realm) -> Result<(), Error> {
        if self.is_private {
            if let Some(instance) = this.to_object()?.downcast::<crate::ClassInstance>() {
                let crate::PropertyKey::String(name) = self.key.clone().into() else {
                    return Err(Error::new("Private field name must be a string"));
                };
                instance.define_private_field(name.to_string(), self.value.copy());
            }
        } else {
            this.define_property(self.key.clone(), self.value.copy(), realm)?;
        }
        Ok(())
    }
}
