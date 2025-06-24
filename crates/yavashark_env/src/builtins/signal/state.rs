use std::cell::RefCell;
use yavashark_garbage::{Weak};
use yavashark_macro::{object, props};
use yavashark_value::{BoxedObj, Obj};
use crate::{MutObject, ObjectHandle, Realm, Res, Value};
use crate::builtins::signal::notify_dependent;

#[object]
#[derive(Debug)]
pub struct State {
    #[mutable]
    pub value: Value,

    #[mutable]
    pub dependents: Vec<Weak<BoxedObj<Realm>>>, //TODO: this should be Vec<Weak<Computed>> or maybe even Vec<Weak<dyn Signal>> in the future
}

impl State {
    pub fn new(value: Value, realm: &Realm) -> Self {
        Self {
            inner: RefCell::new(MutableState {
                object: MutObject::with_proto(realm.intrinsics.signal_state.clone().into()),
                value,
                dependents: Vec::new(),
            }),
        }
    }
}

#[props]
impl State {
    #[constructor]
    pub fn construct(value: Value, _options: Option<ObjectHandle>, realm: &Realm) -> ObjectHandle {
        let state = Self::new(value, realm);

        state.into_object()
    }

    pub fn get(&self) -> Res<Value> {
        let inner = self.inner.try_borrow()?;

        Ok(inner.value.clone())
    }

    pub fn set(&self, value: Value, realm: &mut Realm) -> Res<()> {
        let mut inner = self.inner.try_borrow_mut()?;
        
        inner.value = value;
        
        let mut err = None;
        
        inner.dependents.retain(|dep| {
            if err.is_some() {
                return false;
            }
            
            
            dep.upgrade().is_some_and(|dep| {
                if let Err(error) = notify_dependent(dep, realm) {
                    err = Some(error);
                }

                true
            })
        });
        
        err.map_or(Ok(()), Err)
    }
}