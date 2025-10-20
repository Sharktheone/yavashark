use crate::builtins::signal::computed::Computed;
use crate::builtins::signal::notify_dependent;
use crate::value::{BoxedObj, Obj};
use crate::{MutObject, ObjectHandle, Realm, Res, Value};
use std::cell::RefCell;
use yavashark_garbage::Weak;
use yavashark_macro::{object, props};

#[object]
#[derive(Debug)]
pub struct State {
    #[mutable]
    pub value: Value,

    #[mutable]
    pub dependents: Vec<Weak<BoxedObj>>, //TODO: this should be Vec<Weak<Computed>> or maybe even Vec<Weak<dyn Signal>> in the future
}

impl State {
    pub fn new(value: Value, realm: &mut Realm) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutableState {
                object: MutObject::with_proto(realm.intrinsics.clone_public().signal_state.get(realm)?.clone()),
                value,
                dependents: Vec::new(),
            }),
        })
    }
}

#[props(intrinsic_name = signal_state)]
impl State {
    #[constructor]
    pub fn construct(value: Value, _options: Option<ObjectHandle>, realm: &mut Realm) -> Res<ObjectHandle> {
        let state = Self::new(value, realm)?;

        Ok(state.into_object())
    }

    pub fn get(&self, realm: &mut Realm, this: Value) -> Res<Value> {
        let computed_proto = Computed::get_proto(realm)?;

        if let Some(current_dep) = &*computed_proto.current_dep.borrow() {
            let weak = current_dep.gc().downgrade();

            let mut inner = self.inner.try_borrow_mut()?;

            if !inner.dependents.iter().any(|d| d.ptr_eq(&weak)) {
                inner.dependents.push(weak);
            }

            return Ok(inner.value.clone());
        }

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
                if let Err(error) = notify_dependent(&dep.into(), realm) {
                    err = Some(error);
                }

                true
            })
        });

        err.map_or(Ok(()), Err)
    }
}
