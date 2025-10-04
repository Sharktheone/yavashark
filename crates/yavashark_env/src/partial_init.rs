use std::cell::UnsafeCell;
use crate::{Error, Realm, Res};

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
    
    pub fn get_option(&self) -> Option<&T> {
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