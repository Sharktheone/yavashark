use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use yavashark_value::{Error, MutObj, ObjectImpl};
use crate::{MutObject, Realm, Res, Value, ObjectProperty, Variable};

#[derive(Debug)]
pub struct Arguments {
    pub inner: RefCell<MutObject>,
    pub callee: Value,
    pub length: RefCell<Value>,
    pub args: RefCell<Vec<Value>>,
}


impl Arguments {
    #[must_use] pub fn new(args: Vec<Value>, callee: Value, realm: &Realm) -> Self {
        Self {
            inner: RefCell::new(MutObject::new(realm)),
            callee,
            length: RefCell::new(args.len().into()),
            args: RefCell::new(args),
        }
    }

    pub fn resolve_array(&self, idx: usize) -> Option<ObjectProperty> {
        Some(self.args.borrow().get(idx)?.copy().into())
    }

    pub fn set_array(&self, idx: usize, value: Value) -> Res<()> {
        if let Some(v) = self.args.borrow_mut().get_mut(idx) {
            *v = value;
            return Ok(());
        }
        Err(Error::new("Index out of bounds"))
    }
}


impl ObjectImpl<Realm> for Arguments {
    type Inner = MutObject;

    fn get_wrapped_object(&self) -> impl DerefMut<Target=impl MutObj<Realm>> {
        self.inner.borrow_mut()
    }

    fn get_inner(&self) -> impl Deref<Target=Self::Inner> {
        self.inner.borrow()
    }

    fn get_inner_mut(&self) -> impl DerefMut<Target=Self::Inner> {
        self.inner.borrow_mut()
    }


    fn define_property(&self, name: Value, value: Value) -> Res<()> {
        if let Value::Number(idx) = &name {
            if let Some(v) = self.args.borrow_mut().get_mut(*idx as usize) {
                *v = value;
                return Ok(());
            }
        }
        
        if let Value::String(s) = &name {
            if s == "length" {
                *self.length.borrow_mut() = value;
                return Ok(());
            }
        }

        self.get_wrapped_object().define_property(name, value)
    }

    fn define_variable(&self, name: Value, value: Variable) -> Res<()> {
        if let Value::Number(idx) = &name {
            if let Some(v) = self.args.borrow_mut().get_mut(*idx as usize) {
                *v = value.value;
                return Ok(());
            }
        }
        
        if let Value::String(s) = &name {
            if s == "length" {
                *self.length.borrow_mut() = value.value;
                return Ok(());
            }
        }

        self.get_wrapped_object().define_variable(name, value)
    }

    fn resolve_property(&self, name: &Value) -> Res<Option<ObjectProperty>> {
        if let Value::Number(idx) = &name {
            if let Some(value) = self.resolve_array(*idx as usize) {
                return Ok(Some(value));
            }
        }

        if let Value::String(s) = &name {
            if s == "length" {
                return Ok(Some(self.length.borrow().clone().into()));
            }
            if s == "callee" {
                return Ok(Some(self.callee.clone().into()));
            }
        }

        self.get_wrapped_object().resolve_property(name)

    }

    fn get_property(&self, name: &Value) -> Res<Option<ObjectProperty>> {
        if let Value::Number(idx) = &name {
            if let Some(value) = self.resolve_array(*idx as usize) {
                return Ok(Some(value));
            }
        }

        if let Value::String(s) = &name {
            if s == "length" {
                return Ok(Some(self.length.borrow().clone().into()));
            }
            if s == "callee" {
                return Ok(Some(self.callee.clone().into()));
            }
        }

        self.get_wrapped_object().get_property(name)
    }
}