use std::collections::HashMap;

use crate::{Error, Object, ObjectProperty, Value, ValueResult};
use yavashark_macro::{object, properties};
use yavashark_value::{Constructor, CustomName, Func, Obj};
use crate::realm::Realm;

#[object(function, constructor, direct(prototype))]
#[derive(Debug)]
pub struct Class {
    pub private_props: HashMap<String, Value>,
    pub name: String,
}

impl Func<Realm> for Class {
    fn call(
        &mut self,
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
    fn get_constructor(&self) -> ObjectProperty {
        if let Value::Object(o) = self.prototype.value.copy() {
            o.get_constructor()
        } else {
            self.object.constructor()
        }
    }

    fn value(&self, _realm: &mut Realm) -> Value {
        Object::raw_with_proto(self.prototype.value.clone()).into_value()
    }
}

impl Class {
    #[must_use]
    pub fn new(realm: &Realm, name: String) -> Self {
        Self::new_with_proto(realm.intrinsics.func.clone().into(), name)
    }

    #[must_use]
    pub fn new_with_proto(proto: Value, name: String) -> Self {
        let object = Object::raw_with_proto(proto);

        Self {
            object,
            private_props: HashMap::new(),
            prototype: Value::Undefined.into(),
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

    pub fn set_proto(&mut self, proto: ObjectProperty) {
        self.prototype = proto;
    }
}

#[properties]
impl Class {
    #[constructor(raw)]
    pub fn construct(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
        if let Value::Object(o) = this.copy() {
            let deez = o.get()?;
            let constructor = deez.constructor();
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
            private_props: HashMap::new(),
            object: Object::raw(realm),
            name,
        }
    }
    #[must_use]
    pub fn new_with_proto(proto: Value, name: String) -> Self {
        Self {
            private_props: HashMap::new(),
            object: Object::raw_with_proto(proto),
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
