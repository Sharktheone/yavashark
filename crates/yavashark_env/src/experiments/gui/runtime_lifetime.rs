use crate::{Error, Res};
use std::cell::Cell;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::rc::Rc;

type Internal<T> = (Cell<Option<NonNull<T>>>, Cell<bool>);

pub struct RuntimeLifetime<T>(Rc<Internal<T>>);

pub struct RuntimeLifetimeGuard<'a, T> {
    ptr: Rc<Internal<T>>,
    old: Option<NonNull<T>>,
    borrowed: bool,
    _marker: PhantomData<&'a mut T>,
}

impl<T> Drop for RuntimeLifetimeGuard<'_, T> {
    fn drop(&mut self) {
        assert!(
            !self.ptr.1.get(),
            "Cannot remove reference that is still borrowed"
        );

        self.ptr.0.set(self.old);
        self.ptr.1.set(self.borrowed);
    }
}

impl<T> RuntimeLifetime<T> {
    #[allow(mismatched_lifetime_syntaxes, dead_code)]
    pub fn new(r: &mut T) -> (Self, RuntimeLifetimeGuard<'_, T>) {
        let rc = Rc::new((Cell::new(Some(NonNull::from(r))), Cell::new(false)));

        (
            Self(Rc::clone(&rc)),
            RuntimeLifetimeGuard {
                ptr: rc,
                old: None,
                borrowed: false,
                _marker: PhantomData,
            },
        )
    }

    pub fn empty() -> Self {
        Self(Rc::default())
    }

    pub fn with<R>(&self, f: impl FnOnce(&mut T) -> Res<R>) -> Res<R> {
        if self.0 .1.get() {
            return Err(Error::new("Value already borrowed"));
        }

        self.0 .1.set(true);

        let r = self.0 .0.get();

        let Some(mut r) = r else {
            return Err(Error::new("Used value outside of its context"));
        };

        let r = unsafe { r.as_mut() };

        let res = f(r);

        self.0 .1.set(false);

        res
    }

    pub fn update<'a>(&self, new: &'a mut T) -> RuntimeLifetimeGuard<'a, T> {
        let old = self.0 .0.replace(Some(NonNull::from(new)));
        let borrowed = self.0 .1.replace(false);

        RuntimeLifetimeGuard {
            ptr: self.0.clone(),
            old,
            borrowed,
            _marker: PhantomData,
        }
    }
}
