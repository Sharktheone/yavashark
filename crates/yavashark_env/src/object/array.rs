#![allow(clippy::needless_pass_by_value)]

use std::fmt::Display;

use yavashark_macro::{object, properties};
use yavashark_value::Obj;

use crate::{Context, Error, ObjectHandle, Value, ValueResult, Variable};
use crate::object::Object;
use crate::Symbol;

#[object(direct(length), to_string)]
#[derive(Debug)]
pub struct Array {}


impl Array {
    pub fn with_elements(ctx: &Context, elements: Vec<Value>) -> Result<Self, Error> {
        let mut array = Self::new(ctx.proto.array.clone().into());

        array.object.set_array(elements);

        Ok(array)
    }

    #[must_use]
    pub fn new(proto: Value) -> Self {
        Self {
            object: Object::raw_with_proto(proto),
            length: Variable::new(Value::Number(0.0)),
        }
    }

    #[must_use]
    pub fn from_ctx(ctx: &Context) -> Self {
        Self::new(ctx.proto.array.clone().into())
    }


    pub fn to_string(&self) -> Result<String, Error> {
        let mut buf = String::new();


        for (_, value) in self.object.array.iter() {
            buf.push_str(&value.value.to_string()?);
            buf.push_str(", ");
        }
        
        buf.truncate(buf.len() - 2);
        
        
        Ok(buf)
    }
}

#[properties]
impl Array {
    #[new]
    #[must_use]
    pub fn create(_: &mut Context, proto: &Value) -> Value {
        let this = Self::new(proto.copy());

        ObjectHandle::new(this).into()
    }

    pub fn push(&mut self, value: Value) {
        let index = self.object.array.last().map_or(0, |(i, _)| *i + 1);

        self.object.array.push((index, Variable::new(value)));
        self.length.value = Value::Number(index as f64 + 1.0);
    }

    #[prop(Symbol::ITERATOR)]
    #[allow(clippy::unused_self)]
    fn iterator(&self, _args: Vec<Value>, ctx: &Context, this: Value) -> ValueResult {
        let Value::Object(obj) = this else {
            return Err(Error::ty_error(format!("Expected object, found {this:?}")));
        };

        let iter = ArrayIterator {
            object: Object::raw_with_proto(ctx.proto.array_iter.clone().into()),
            inner: obj,
            next: 0,
            done: false,
        };

        let iter: Box<dyn Obj<Context>> = Box::new(iter);

        Ok(iter.into())
    }

    #[constructor(special)]
    fn construct(&mut self, args: Vec<Value>) -> ValueResult {
        let values = args
            .into_iter()
            .map(Variable::new)
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
    pub fn next(&mut self, _args: Vec<Value>, ctx: &Context) -> ValueResult {
        if self.done {
            let obj = Object::new(ctx);
            obj.define_property("value".into(), Value::Undefined)?;
            obj.define_property("done".into(), Value::Boolean(true))?;
            return Ok(obj.into());
        }

        let inner = self.inner.get()?;

        let (done, value) = inner.get_array_or_done(self.next);

        self.next += 1;

        if done {
            self.done = true;
            let obj = Object::new(ctx);
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

        let obj = Object::new(ctx);
        obj.define_property("value".into(), value)?;
        obj.define_property("done".into(), Value::Boolean(self.done))?;

        Ok(obj.into())
    }
}
