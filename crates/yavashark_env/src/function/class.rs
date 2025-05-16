use std::cell::RefCell;
use std::collections::HashMap;

use crate::realm::Realm;
use crate::{Error, MutObject, Object, ObjectProperty, Res, Value, ValueResult};
use yavashark_macro::{object, properties};
use yavashark_value::{Constructor, ConstructorFn, CustomName, Func, NoOpConstructorFn, Obj};

#[object(function, constructor, direct(prototype))]
#[derive(Debug)]
pub struct Class {
    pub private_props: HashMap<String, Value>,
    pub name: String,
    #[gc(untyped)]
    pub constructor: Box<dyn ConstructorFn<Realm>>,
}

impl Func<Realm> for Class {
    fn call(&self, _realm: &mut Realm, _args: Vec<Value>, _this: Value) -> Res<Value, Error> {
        Err(Error::new(
            "Class constructor cannot be invoked without 'new'",
        ))
    }
}

impl Constructor<Realm> for Class {
    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> ValueResult {
        let inner = self.inner.try_borrow()?;

        let this = ClassInstance::new_with_proto(inner.prototype.value.clone(), self.name.clone()).into_value();

        drop(inner);

        self.constructor.construct(args, this.copy(), realm)?;

        Ok(this)
    }

    fn construct_proto(&self) -> Res<ObjectProperty> {
        let inner = self.inner.try_borrow()?;

        Ok(inner.prototype.clone())
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
            constructor: Box::new(NoOpConstructorFn),
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

    pub fn set_proto(&mut self, proto: ObjectProperty) -> Res<(), Error> {
        let mut inner = self.inner.try_borrow_mut()?;
        inner.prototype = proto;

        Ok(())
    }

    pub fn set_constructor(&mut self, constructor: impl ConstructorFn<Realm> + 'static) {
        self.constructor = Box::new(constructor);
    }
}

#[properties]
impl Class {
    #[constructor(raw)]
    pub fn construct(args: Vec<Value>, realm: &mut Realm) -> ValueResult {
        let this = Self::new(realm, "Class".to_string()).into_value();

        if let Value::Object(o) = this.copy() {
            let deez = o.guard();
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
    #[mutable]
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
                private_props: HashMap::new(),
            }),
            name,
        }
    }
    #[must_use]
    pub fn new_with_proto(proto: Value, name: String) -> Self {
        Self {
            inner: RefCell::new(MutableClassInstance {
                object: MutObject::with_proto(proto),
                private_props: HashMap::new(),
            }),
            name,
        }
    }

    pub fn set_private_prop(&mut self, key: String, value: Value) {
        self.inner.get_mut().private_props.insert(key, value);
    }

    pub fn get_private_prop(&self, key: &str) -> Res<Option<Value>> {
        let inner = self.inner.try_borrow()?;

        let mut prop = inner.private_props.get(key).cloned();

        if prop.is_none() {
            let proto = inner.object.prototype.value.clone();

            drop(inner);

            if let Some(class) = proto.downcast::<Self>()? {
                prop = class.get_private_prop(key)?;
            }
        }

        Ok(prop)
    }
}
