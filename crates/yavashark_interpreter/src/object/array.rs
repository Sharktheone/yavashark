use yavashark_macro::{object, properties};

use crate::object::Object;
use crate::Symbol;
use crate::{Context, Error, NativeFunction, ObjectHandle, Value, ValueResult, Variable};

#[derive(Debug)]
#[object(direct(iter(Symbol::ITERATOR)))]
pub struct Array {}

#[properties]
impl Array {
    #[prop]
    fn length(&self, args: Vec<Value>) -> ValueResult {
        todo!()
    }

    #[prop(Symbol::ITERATOR)]
    fn iterator(&self, args: Vec<Value>) -> ValueResult {
        todo!()
    }

    #[constructor]
    fn construct(&self, args: Vec<Value>) -> ValueResult {
        todo!()
    }
}

#[derive(Debug)]
#[object]
pub struct ArrayIterator {}

impl From<Vec<Value>> for Array {
    fn from(v: Vec<Value>) -> Self {
        todo!()
    }
}
