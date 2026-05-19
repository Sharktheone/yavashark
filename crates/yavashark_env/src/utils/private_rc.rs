use std::ops::{Deref, DerefMut};
use std::rc::Rc;

pub struct PrivateRc<T>(Rc<T>);

impl<T> Deref for PrivateRc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> PrivateRc<T> {
    pub fn new(value: T) -> Self {
        Self(Rc::new(value))
    }
}

impl<T> DerefMut for PrivateRc<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Rc::get_mut(&mut self.0).expect("Multiple references exist")
    }
}

impl<T> PrivateRc<T> {
    #[must_use]
    pub fn clone_public<'a>(&self) -> PublicRc<'a, T> {
        PublicRc(self.0.clone(), std::marker::PhantomData)
    }
}

pub struct PublicRc<'a, T>(Rc<T>, std::marker::PhantomData<&'a ()>);

impl<'a, T> Deref for PublicRc<'a, T>
where
    T: 'a,
    Self: 'a,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
