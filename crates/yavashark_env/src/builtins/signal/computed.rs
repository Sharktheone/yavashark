use crate::{ObjectHandle, Realm, Value, ValueResult};
use std::cell::Cell;
use yavashark_garbage::Weak;
use yavashark_macro::{object, props};
use yavashark_value::BoxedObj;
use crate::builtins::signal::notify_dependent;

#[object]
#[derive(Debug)]
pub struct Computed {
    #[mutable]
    pub value: Value,

    pub compute_fn: ObjectHandle,

    pub dirty: Cell<bool>,
    pub dependents: Vec<Weak<BoxedObj<Realm>>>, //TODO: this should be Vec<Weak<Computed>> or maybe even Vec<Weak<dyn Signal>> in the future
}

#[props]
impl Computed {
    
    #[constructor]
    pub fn construct() -> ObjectHandle {
        todo!()
    }
    
    pub fn get(&self, realm: &mut Realm) -> ValueResult {
        if self.dirty.get() {
            self.dirty.set(false);
            
            let new = self.compute_fn.call(realm, Vec::new(), Value::Undefined)?;
            
            let mut inner = self.inner.try_borrow_mut()?;
            
            inner.value = new;
            
            // TODO: what to do if the value is the same?
            
            drop(inner);
            
            for dep in &self.dependents {
                if let Some(dep) = dep.upgrade() {
                    notify_dependent(&dep.into(), realm)?;
                }
            }
            
        }
        
        
        let inner = self.inner.try_borrow()?;
        
        Ok(inner.value.clone())
    }
    
}
