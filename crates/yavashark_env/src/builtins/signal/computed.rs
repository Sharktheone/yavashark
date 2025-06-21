use std::cell::Cell;
use yavashark_garbage::Weak;
use yavashark_macro::{object, props};
use yavashark_value::BoxedObj;
use crate::{ObjectHandle, Realm, Value};

#[object]
#[derive(Debug)]
pub struct Computed {
    #[mutable]
    pub value: Value,
    
    pub compute_fn: ObjectHandle,
    
    pub dirty: Cell<bool>,
    pub dependents: Vec<Weak<BoxedObj<Realm>>> //TODO: this should be Vec<Weak<Computed>> or maybe even Vec<Weak<dyn Signal>> in the future
    
}

#[props]
impl Computed {}