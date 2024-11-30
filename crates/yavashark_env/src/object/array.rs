#![allow(clippy::needless_pass_by_value)]

use yavashark_macro::{object, properties};
use yavashark_value::Obj;

use crate::object::Object;
use crate::realm::Realm;
use crate::ObjectProperty;
use crate::{Error, ObjectHandle, Value, ValueResult, Variable};

#[object(direct(length), to_string)]
#[derive(Debug)]
pub struct Array {}

impl Array {
    pub fn with_elements(realm: &Realm, elements: Vec<Value>) -> Result<Self, Error> {
        let mut array = Self::new(realm.intrinsics.array.clone().into());

        array.object.set_array(elements);
        array.length.value = Value::Number(array.object.array.len() as f64);

        Ok(array)
    }

    #[must_use]
    pub fn new(proto: Value) -> Self {
        Self {
            object: Object::raw_with_proto(proto),
            length: ObjectProperty::new(Value::Number(0.0)),
        }
    }

    #[must_use]
    pub fn from_realm(realm: &Realm) -> Self {
        Self::new(realm.intrinsics.array.clone().into())
    }

    pub fn override_to_string(&self, realm: &mut Realm) -> Result<String, Error> {
        let mut buf = String::new();

        for (_, value) in &self.object.array {
            buf.push_str(&value.value.to_string(realm)?);
            buf.push_str(", ");
        }

        buf.pop();
        buf.pop();

        Ok(buf)
    }

    #[must_use]
    pub fn override_to_string_internal(&self) -> String {
        use std::fmt::Write as _;

        let mut buf = String::new();

        for (_, value) in &self.object.array {
            let _ = write!(buf, "{}", value.value);

            buf.push_str(", ");
        }

        buf.pop();
        buf.pop();

        buf
    }
}

#[properties]
impl Array {
    #[new]
    #[must_use]
    pub fn create(_: &mut Realm, proto: &Value) -> Value {
        let this = Self::new(proto.copy());

        ObjectHandle::new(this).into()
    }

    pub fn push(&mut self, value: Value) {
        let index = self.object.array.last().map_or(0, |(i, _)| *i + 1);

        self.object.array.push((index, Variable::new(value).into()));
        self.length.value = Value::Number(index as f64 + 1.0);
    }

    #[prop(crate::Symbol::ITERATOR)]
    #[allow(clippy::unused_self)]
    fn iterator(&self, _args: Vec<Value>, realm: &Realm, this: Value) -> ValueResult {
        let Value::Object(obj) = this else {
            return Err(Error::ty_error(format!("Expected object, found {this:?}")));
        };

        let iter = ArrayIterator {
            object: Object::raw_with_proto(realm.intrinsics.array_iter.clone().into()),
            inner: obj,
            next: 0,
            done: false,
        };

        let iter: Box<dyn Obj<Realm>> = Box::new(iter);

        Ok(iter.into())
    }

    #[constructor(special)]
    fn construct(&mut self, args: Vec<Value>) -> ValueResult {
        let values = args
            .into_iter()
            .map(ObjectProperty::new)
            .enumerate()
            .collect::<Vec<_>>();
        self.object.array = values;
        self.length.value = Value::Number(self.object.array.len() as f64);

        Ok(Value::Undefined)
    }
}

#[object]
#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct ArrayIterator {
    inner: ObjectHandle,
    next: usize,
    done: bool,
}

#[properties]
impl ArrayIterator {
    #[prop]
    pub fn next(&mut self, _args: Vec<Value>, realm: &Realm) -> ValueResult {
        if self.done {
            let obj = Object::new(realm);
            obj.define_property("value".into(), Value::Undefined)?;
            obj.define_property("done".into(), Value::Boolean(true))?;
            return Ok(obj.into());
        }

        let inner = self.inner.get()?;

        let (done, value) = inner.get_array_or_done(self.next);

        self.next += 1;

        if done {
            self.done = true;
            let obj = Object::new(realm);
            obj.define_property("value".into(), Value::Undefined)?;
            obj.define_property("done".into(), Value::Boolean(true))?;
            return Ok(obj.into());
        }

        let value = if let Some(value) = value {
            value
        } else {
            self.done = true;
            Value::Undefined
        };

        let obj = Object::new(realm);
        obj.define_property("value".into(), value)?;
        obj.define_property("done".into(), Value::Boolean(self.done))?;

        Ok(obj.into())
    }
}
