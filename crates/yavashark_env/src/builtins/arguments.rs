use crate::{MutObject, ObjectProperty, Realm, Res, Value, ValueResult, Variable};
use std::cell::{Cell, RefCell};
use std::ops::{Deref, DerefMut};
use yavashark_macro::props;
use yavashark_string::YSString;
use yavashark_value::{Error, MutObj, Obj, ObjectImpl};
use crate::array::{ArrayIterator, MutableArrayIterator};

#[derive(Debug)]
pub struct Arguments {
    pub inner: RefCell<MutObject>,
    pub callee: Value,
    pub length: RefCell<Value>,
    pub args: RefCell<Vec<Value>>,
}

impl Arguments {
    #[must_use]
    pub fn new(args: Vec<Value>, callee: Value, realm: &Realm) -> Self {
        Self {
            inner: RefCell::new(MutObject::with_proto(realm.intrinsics.arguments.clone().into())),
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

    fn get_wrapped_object(&self) -> impl DerefMut<Target = impl MutObj<Realm>> {
        self.inner.borrow_mut()
    }

    fn get_inner(&self) -> impl Deref<Target = Self::Inner> {
        self.inner.borrow()
    }

    fn get_inner_mut(&self) -> impl DerefMut<Target = Self::Inner> {
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

    fn name(&self) -> String {
        "Arguments".to_string()
    }

    fn to_string(&self, _: &mut Realm) -> Result<YSString, Error<Realm>> {
        Ok("[object Arguments]".into())
    }

    fn to_string_internal(&self) -> Result<YSString, Error<Realm>> {
        Ok("[object Arguments]".into())
    }

    fn get_array_or_done(&self, index: usize) -> Result<(bool, Option<Value>), Error<Realm>> {
        let args = self.args.borrow();
        if index < args.len() {
            Ok((true, Some(args[index].clone())))
        } else {
            Ok((false, None))
        }
    }
}


#[props]
impl Arguments {
    #[prop(crate::Symbol::ITERATOR)]
    #[nonstatic]
    fn iterator(realm: &Realm, this: Value) -> ValueResult {
        let Value::Object(obj) = this else {
            return Err(crate::Error::ty_error(format!("Expected object, found {this:?}")));
        };
        
        let iter = ArrayIterator {
            inner: RefCell::new(MutableArrayIterator {
                object: MutObject::with_proto(realm.intrinsics.array_iter.clone().into()),
            }),
            array: obj,
            next: Cell::new(0),
            done: Cell::new(false),
        };

        let iter: Box<dyn Obj<Realm>> = Box::new(iter);

        Ok(iter.into())
    }
}