#![allow(clippy::needless_pass_by_value)]

use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use yavashark_macro::{object, properties};
use yavashark_value::Iter;

use crate::{Context, Error, Value, ValueResult, Variable};
use crate::object::Object;
use crate::Symbol;

#[derive(Debug)]
#[object]
pub struct Array {}


#[properties]
impl Array {
    #[new]
    #[allow(clippy::unnecessary_wraps)]
    fn new(ctx: &mut Context) -> Result<Self, Error> {
        Ok(Self {
            object: Object::raw_with_proto(ctx.proto.array_prototype.clone().into())
        })
    }

    #[prop]
    fn length(&self, args: Vec<Value>) -> ValueResult {
        todo!()
    }

    #[prop(Symbol::ITERATOR)]
    fn iterator(&self, args: Vec<Value>) -> ValueResult {
        todo!()
    }

    #[constructor]
    #[allow(clippy::unnecessary_wraps)]
    fn construct(&mut self, args: Vec<Value>) -> ValueResult {
        let values = args.into_iter().enumerate()
            .map(|(i, v)| {
                (i, Variable::new(v))
            })
            .collect::<Vec<_>>();
        self.object.array = values;


        Ok(Value::Undefined)
    }
}

#[derive(Debug)]
#[object]
#[allow(clippy::module_name_repetitions)]
pub struct ArrayIterator {
    inner: Rc<RefCell<Array>>,
    next: usize,
    done: bool
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
        
        let value = self.inner.get_array_or_done(self.next).map(|v| (v.0.copy(), v.1));
        self.next += 1;
        
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
