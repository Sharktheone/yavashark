#![allow(clippy::needless_pass_by_value)]

use yavashark_macro::{object, properties};
use yavashark_value::Obj;

use crate::object::Object;
use crate::Symbol;
use crate::{Context, Error, ObjectHandle, Value, ValueResult, Variable};

#[object(direct(length))]
#[derive(Debug)]
pub struct Array {}

#[properties]
impl Array {
    #[new]
    pub fn new(ctx: &mut Context) -> Result<Self, Error> {
        Ok(Self {
            object: Object::raw_with_proto(ctx.proto.array.clone().into()),
            length: Value::Number(0.0).into(),
        })
    }

    pub fn push(&mut self, value: Value) {
        let index = self.object.array.last().map(|(i, _)| *i + 1).unwrap_or(0);

        self.object.array.push((index, Variable::new(value)));
        self.length.value = Value::Number(index as f64 + 1.0);
    }

    #[prop(Symbol::ITERATOR)]
    fn iterator(&self, args: Vec<Value>, ctx: &mut Context, this: Value) -> ValueResult {
        let obj = match this {
            Value::Object(obj) => obj,

            Value::Function(func) => {
                todo!("Function iterator")
            }

            _ => return Err(Error::ty_error(format!("Expected object, found {this:?}"))),
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

    #[constructor]
    fn construct(&mut self, args: Vec<Value>) -> ValueResult {
        let values = args
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i, Variable::new(v)))
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
    pub fn next(&mut self, args: Vec<Value>, ctx: &mut Context) -> ValueResult {
        if self.done {
            let obj = Object::new(ctx);
            obj.define_property("value".into(), Value::Undefined);
            obj.define_property("done".into(), Value::Boolean(true));
            return Ok(obj.into());
        }

        let inner = self.inner.get()?;

        let (done, value) = inner.get_array_or_done(self.next);

        self.next += 1;

        if done {
            self.done = true;
            let obj = Object::new(ctx);
            obj.define_property("value".into(), Value::Undefined);
            obj.define_property("done".into(), Value::Boolean(true));
            return Ok(obj.into());
        }

        let value = if let Some(value) = value {
            value
        } else {
            self.done = true;
            Value::Undefined
        };

        let obj = Object::new(ctx);
        obj.define_property("value".into(), value);
        obj.define_property("done".into(), Value::Boolean(self.done));

        Ok(obj.into())
    }
}

impl From<Vec<Value>> for Array {
    fn from(v: Vec<Value>) -> Self {
        todo!()
    }
}
