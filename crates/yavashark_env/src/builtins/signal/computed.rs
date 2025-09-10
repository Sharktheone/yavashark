use crate::builtins::signal::notify_dependent;
use crate::{ControlFlow, GCd, Object};
use crate::{Error, MutObject, ObjectHandle, Realm, Res, Value, ValueResult};
use std::cell::{Cell, RefCell};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use yavashark_garbage::{Gc, OwningGcGuard, Weak};
use yavashark_macro::{object, props};
use yavashark_value::{BoxedObj, MutObj, Obj};

#[object]
#[derive(Debug)]
pub struct Computed {
    #[mutable]
    pub value: Value,

    pub compute_fn: ObjectHandle,

    pub dirty: Cell<bool>,
    #[mutable]
    pub dependents: Vec<Weak<BoxedObj<Realm>>>, //TODO: this should be Vec<Weak<Computed>> or maybe even Vec<Weak<dyn Signal>> in the future
}

impl Computed {
    pub fn new(compute_fn: ObjectHandle, realm: &Realm) -> Res<Self> {
        if !compute_fn.is_function() {
            return Err(Error::ty(
                "Computed constructor expects a function as the first argument",
            ));
        }

        Ok(Self {
            inner: RefCell::new(MutableComputed {
                object: MutObject::with_proto(realm.intrinsics.signal_computed.clone().into()),
                value: Value::Undefined,
                dependents: Vec::new(),
            }),

            compute_fn,
            dirty: Cell::new(true),
        })
    }

    pub fn get_proto(realm: &Realm) -> Res<GCd<ComputedProtoObj>> {
        let proto = &realm.intrinsics.signal_computed;

        proto
            .downcast::<ComputedProtoObj>()
            .ok_or_else(|| Error::ty("Computed prototype is not a ComputedProtoObj"))
    }

    fn setup_dependency_tracking(realm: &mut Realm, this: &ObjectHandle) -> Res<Option<GCd<Self>>> {
        let p = Self::get_proto(realm)?;

        let mut dep = p.current_dep.borrow_mut();

        let old = dep.take();

        let new = this
            .downcast::<Self>()
            .ok_or_else(|| Error::ty("Computed.get called on non-Computed object"))?;

        //TODO: we somehow also need to remove this from all dependencies of the old computed

        *dep = Some(new);

        Ok(old)
    }

    fn restore_dependency_tracking(realm: &mut Realm, old: Option<GCd<Self>>) -> Res<()> {
        let p = Self::get_proto(realm)?;

        let mut dep = p.current_dep.borrow_mut();

        *dep = old;

        Ok(())
    }

    pub fn add_dependent(&self, dependent: &ObjectHandle) {
        let mut inner = self.inner.borrow_mut();

        let weak = Gc::downgrade(dependent);

        inner.dependents.push(weak);

        drop(inner);
    }
}

#[derive(Debug)]
pub struct ComputedProtoObj {
    pub(crate) obj: Object,
    pub(crate) current_dep: RefCell<Option<GCd<Computed>>>,
}

impl Deref for ComputedProtoObj {
    type Target = Object;

    fn deref(&self) -> &Self::Target {
        &self.obj
    }
}

impl yavashark_value::ObjectImpl<Realm> for ComputedProtoObj {
    type Inner = Option<GCd<Computed>>;

    fn get_wrapped_object(&self) -> impl DerefMut<Target = impl MutObj<Realm>> {
        self.obj
            .inner_mut()
            .expect("TODO: handle this case properly")
    }

    fn get_inner(&self) -> impl Deref<Target = Self::Inner> {
        self.current_dep.borrow()
    }

    fn get_inner_mut(&self) -> impl DerefMut<Target = Self::Inner> {
        self.current_dep.borrow_mut()
    }
}

#[props(override_object=ComputedProtoObj)]
impl Computed {
    #[constructor]
    pub fn construct(cb: ObjectHandle, realm: &Realm) -> Res<ObjectHandle> {
        let obj = Self::new(cb, realm)?;

        Ok(obj.into_object())
    }

    pub fn get(&self, realm: &mut Realm, this: Value) -> ValueResult {
        if self.dirty.replace(false) {
            let this = this.as_object()?;

            let old = Self::setup_dependency_tracking(realm, this)?;

            if let Some(old) = &old {
                old.add_dependent(this);
            }

            let new = self.compute_fn.call(realm, Vec::new(), Value::Undefined)?;
            Self::restore_dependency_tracking(realm, old)?;

            let mut inner = self.inner.try_borrow_mut()?;

            inner.value = new;

            // TODO: what to do if the value is the same?

            for dep in &inner.dependents {
                if let Some(dep) = dep.upgrade() {
                    notify_dependent(&dep.into(), realm)?;
                }
            }
        }

        let inner = self.inner.try_borrow()?;

        Ok(inner.value.clone())
    }
}
