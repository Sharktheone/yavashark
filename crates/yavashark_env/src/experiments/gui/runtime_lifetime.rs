use std::cell::RefCell;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::rc::Rc;
use crate::{Error, Res};

pub struct RuntimeLifetime<T>(Rc<RefCell<Option<NonNull<T>>>>);

pub struct RuntimeLifetimeGuard<'a, T>(Rc<RefCell<Option<NonNull<T>>>>, PhantomData<&'a mut T>);

impl<'a, T> Drop for RuntimeLifetimeGuard<'a, T> {
    fn drop(&mut self) {
        *self.0.borrow_mut() = None;
    }
}

impl<T> RuntimeLifetime<T> {
    #[allow(elided_named_lifetimes)]
    pub fn new<'a>(r: &'a mut T) -> (Self, RuntimeLifetimeGuard<'a, T>) {
        let rc = Rc::new(RefCell::new(Some(NonNull::from(r))));
        
        (Self(Rc::clone(&rc)), RuntimeLifetimeGuard(rc, PhantomData))
    }
    
    pub fn empty() -> RuntimeLifetime<T> {
        Self(Rc::default())
    }



    pub fn with<R>(&self, f: impl FnOnce(&mut T) -> Res<R>) -> Res<R> {
        let r = self.0.try_borrow_mut()?;

        let Some(mut r) = *r else {
            return Err(Error::new("Used value outside of its context"))
        };

        let r = unsafe { r.as_mut() };

        f(r)
    }
    
    pub fn update<'a>(&self, new: &'a mut T) -> RuntimeLifetimeGuard<'a, T> {
        *self.0.borrow_mut() = Some(NonNull::from(new));
        
        RuntimeLifetimeGuard(self.0.clone(), PhantomData)
    }
}