#![allow(clippy::needless_pass_by_value)]

use std::any::Any;
use yavashark_macro::{object, properties};

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
pub struct ArrayIterator {}

impl From<Vec<Value>> for Array {
    fn from(v: Vec<Value>) -> Self {
        todo!()
    }
}
