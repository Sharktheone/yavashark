use crate::{Error, MutObject, ObjectHandle, Realm, Res, Value, ValueResult};
use std::cell::{Cell, RefCell};
use yavashark_garbage::Weak;
use yavashark_macro::{object, props};
use yavashark_value::{BoxedObj, Obj};
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


impl Computed {
    pub fn new(compute_fn: ObjectHandle, realm: &Realm) -> Res<Self> {
        if !compute_fn.is_function() {
            return Err(Error::ty(
                "Computed constructor expects a function as the first argument"
            ));
        }
        
        Ok(Self {
            inner: RefCell::new(MutableComputed {
                object: MutObject::with_proto(realm.intrinsics.signal_computed.clone().into()),
                value: Value::Undefined,
            }),
            
            compute_fn,
            dirty: Cell::new(true),
            dependents: Vec::new(),
        })
    }
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
