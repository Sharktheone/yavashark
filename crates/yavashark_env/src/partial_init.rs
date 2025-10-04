use std::borrow::Cow;
use std::cell::{Cell, RefCell, UnsafeCell};
use crate::{Error, Realm, Res, Value};
use crate::conversion::FromValueOutput;

pub struct Partial<T, I: Initializer<T>> {
    value: UnsafeCell<Option<T>>,
    _init: std::marker::PhantomData<I>,
}


impl<T, I: Initializer<T>> Default for Partial<T, I> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, I: Initializer<T>> Partial<T, I> {
    pub const fn new() -> Self {
        Self {
            value: UnsafeCell::new(None),
            _init: std::marker::PhantomData,
        }
    }

    pub fn get(&self, realm: &mut Realm) -> Res<&T> {
        unsafe {
            if (*self.value.get()).is_none() {
                let v = I::initialize(realm)?;
                *self.value.get() = Some(v);
            }

            (*self.value.get()).as_ref()
                .ok_or_else(|| Error::new("Failed to initialize Partial value"))
        }
    }

    pub fn get_opt(&self) -> Option<&T> {
        unsafe {
            (*self.value.get()).as_ref()
        }
    }

    pub fn set(&self, value: T) -> Option<T> {
        unsafe {
            if (*self.value.get()).is_some() {
                Some(value)
            } else {
                *self.value.get() = Some(value);
                None
            }
        }
    }

    pub fn is_initialized(&self) -> bool {
        unsafe {
            (*self.value.get()).is_some()
        }
    }
}


pub trait Initializer<T> {
    fn initialize(realm: &mut Realm) -> Res<T>;
}


impl<T: FromValueOutput, I: Initializer<T>> FromValueOutput for Partial<T, I> {
    type Output = T::Output;
    fn from_value_out(value: Value, realm: &mut Realm) -> Res<Self::Output> {
        T::from_value_out(value, realm)
    }
}


impl<T, I: Initializer<T>> Initializer<RefCell<T>> for I {
    fn initialize(realm: &mut Realm) -> Res<RefCell<T>> {
        let v = I::initialize(realm)?;
        Ok(RefCell::new(v))
    }
}
impl<T, I: Initializer<T>> Initializer<Cell<T>> for I {
    fn initialize(realm: &mut Realm) -> Res<Cell<T>> {
        let v = I::initialize(realm)?;
        Ok(Cell::new(v))
    }
}
