use std::cell::RefCell;
use yavashark_macro::object;
use crate::{MutObject, Realm, Value};

#[object(direct(callee))]
#[derive(Debug)]
pub struct Arguments {
    pub args: Vec<Value>,
}


impl Arguments {
    pub fn new(args: Vec<Value>, callee: Value, realm: &Realm) -> Self {
        Self { 
            inner: RefCell::new(MutableArguments {
                object: MutObject::new(realm),
                callee: callee.into(),
            }),
            args
        }
    }
}