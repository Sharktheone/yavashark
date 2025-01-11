use std::cell::RefCell;
use std::collections::HashMap;

use crate::realm::Realm;
use crate::{Error, MutObject, Object, ObjectProperty, Value, ValueResult};
use yavashark_macro::{object, properties};
use yavashark_value::{Constructor, CustomName, Func, Obj};

#[object(function, constructor, direct(prototype))]
#[derive(Debug)]
pub struct Class {
    pub private_props: HashMap<String, Value>,
    pub name: String,
}

impl Func<Realm> for Class {
    fn call(
        &self,
        _realm: &mut Realm,
        _args: Vec<Value>,
        _this: Value,
    ) -> Result<Value, Error> {
        Err(Error::new(
            "Class constructor cannot be invoked without 'new'",
        ))
    }
}

impl Constructor<Realm> for Class {
    fn get_constructor(&self) -> Result<ObjectProperty, Error> {
        let inner = self.inner.try_borrow().map_err(|_| Error::borrow_error())?;
        
        if let Value::Object(o) = inner.prototype.value.copy() {
            o.constructor()
        } else {
            inner.object.constructor()
        }
    }

    fn value(&self, _realm: &mut Realm) -> ValueResult {
        let inner = self.inner.try_borrow().map_err(|_| Error::borrow_error())?;
        
        Ok(Object::raw_with_proto(inner.prototype.value.clone()).into_value())
    }
}

impl Class {
    #[must_use]
    pub fn new(realm: &Realm, name: String) -> Self {
        Self::new_with_proto(realm.intrinsics.func.clone().into(), name)
    }

    #[must_use]
    pub fn new_with_proto(proto: Value, name: String) -> Self {

        Self {
            inner: RefCell::new(MutableClass {
                object: MutObject::with_proto(proto),
                prototype: Value::Undefined.into(),
            }),
            private_props: HashMap::new(),
            name,
        }
    }

    pub fn set_private_prop(&mut self, key: String, value: Value) {
        self.private_props.insert(key, value);
    }

    #[must_use]
    pub fn get_private_prop(&self, key: &str) -> Option<&Value> {
        self.private_props.get(key)
    }

    pub fn set_proto(&mut self, proto: ObjectProperty) -> Result<(), Error> {
        let mut inner = self.inner.try_borrow_mut().map_err(|_| Error::borrow_error())?;
        inner.prototype = proto;
        
        Ok(())
    }
}

#[properties]
impl Class {
    #[constructor(raw)]
    pub fn construct(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
        if let Value::Object(o) = this.copy() {
            let deez = o.get();
            let constructor = deez.constructor()?;
            drop(deez);
            let constructor = constructor.resolve(Value::Object(o), realm)?;

            constructor.call(realm, args, this)
        } else {
            Err(Error::ty("Class constructor called with invalid receiver"))
        }
    }
}

#[object(name)]
#[derive(Debug)]
pub struct ClassInstance {
    pub(crate) private_props: HashMap<String, Value>,
    name: String,
}

impl CustomName for ClassInstance {
    fn custom_name(&self) -> String {
        self.name.clone()
    }
}

impl ClassInstance {
    #[must_use]
    pub fn new(realm: &Realm, name: String) -> Self {
        Self {
            inner: RefCell::new(MutableClassInstance {
                object: MutObject::new(realm),
            }),
            private_props: HashMap::new(),
            name,
        }
    }
    #[must_use]
    pub fn new_with_proto(proto: Value, name: String) -> Self {
        Self {
            inner: RefCell::new(MutableClassInstance {
                object: MutObject::with_proto(proto),
            }),
            private_props: HashMap::new(),
            name,
        }
    }

    pub fn set_private_prop(&mut self, key: String, value: Value) {
        self.private_props.insert(key, value);
    }

    #[must_use]
    pub fn get_private_prop(&self, key: &str) -> Option<&Value> {
        self.private_props.get(key)
    }
}
