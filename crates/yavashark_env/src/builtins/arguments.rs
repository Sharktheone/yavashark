use crate::array::{ArrayIterator, MutableArrayIterator};
use crate::error::Error;
use crate::value::{Attributes, DefinePropertyResult, MutObj, Obj, ObjectImpl, Property};
use crate::{InternalPropertyKey, MutObject, Realm, Res, Value, ValueResult, Variable};
use std::cell::{Cell, RefCell};
use std::ops::{Deref, DerefMut};
use yavashark_macro::props;

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
            inner: RefCell::new(MutObject::with_proto(realm.intrinsics.arguments.clone())),
            callee,
            length: RefCell::new(args.len().into()),
            args: RefCell::new(args),
        }
    }

    pub fn resolve_array(&self, idx: usize) -> Option<Value> {
        Some(self.args.borrow().get(idx)?.copy())
    }

    pub fn set_array(&self, idx: usize, value: Value) -> Res<()> {
        if let Some(v) = self.args.borrow_mut().get_mut(idx) {
            *v = value;
            return Ok(());
        }
        Err(Error::new("Index out of bounds"))
    }
}

impl ObjectImpl for Arguments {
    type Inner = MutObject;

    fn get_wrapped_object(&self) -> impl DerefMut<Target = impl MutObj> {
        self.inner.borrow_mut()
    }

    fn get_inner(&self) -> impl Deref<Target = Self::Inner> {
        self.inner.borrow()
    }

    fn get_inner_mut(&self) -> impl DerefMut<Target = Self::Inner> {
        self.inner.borrow_mut()
    }

    fn define_property(
        &self,
        name: InternalPropertyKey,
        value: Value,
        realm: &mut Realm,
    ) -> Res<DefinePropertyResult> {
        if let InternalPropertyKey::Index(idx) = name {
            if let Some(v) = self.args.borrow_mut().get_mut(idx) {
                *v = value;
                return Ok(DefinePropertyResult::Handled);
            }
        }

        if let InternalPropertyKey::String(s) = &name {
            if s == "length" {
                *self.length.borrow_mut() = value;
                return Ok(DefinePropertyResult::Handled);
            }
        }

        self.get_wrapped_object()
            .define_property(name, value, realm)
    }

    fn define_property_attributes(
        &self,
        name: InternalPropertyKey,
        value: Variable,
        realm: &mut Realm,
    ) -> Res<DefinePropertyResult> {
        if let InternalPropertyKey::Index(idx) = name {
            if let Some(v) = self.args.borrow_mut().get_mut(idx) {
                *v = value.value;
                return Ok(DefinePropertyResult::Handled);
            }
        }

        if let InternalPropertyKey::String(s) = &name {
            if s == "length" {
                *self.length.borrow_mut() = value.value;
                return Ok(DefinePropertyResult::Handled);
            }
        }

        self.get_wrapped_object()
            .define_property_attributes(name, value, realm)
    }

    fn resolve_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        if let InternalPropertyKey::Index(idx) = name {
            if let Some(value) = self.resolve_array(idx) {
                return Ok(Some(Property::Value(value, Attributes::new())));
            }
        }

        if let InternalPropertyKey::String(s) = &name {
            if s == "length" {
                return Ok(Some(Property::Value(
                    self.length.borrow().clone(),
                    Attributes::write_config(),
                )));
            }
            if s == "callee" {
                return Ok(Some(Property::Value(
                    self.callee.clone(),
                    Attributes::write_config(),
                )));
            }
        }

        self.get_wrapped_object().resolve_property(name, realm)
    }

    fn get_own_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        if let InternalPropertyKey::Index(idx) = name {
            if let Some(value) = self.resolve_array(idx) {
                return Ok(Some(value.into()));
            }
        }

        if let InternalPropertyKey::String(s) = &name {
            if s == "length" {
                return Ok(Some(self.length.borrow().clone().into()));
            }
            if s == "callee" {
                return Ok(Some(self.callee.clone().into()));
            }
        }

        self.get_wrapped_object().get_own_property(name, realm)
    }

    fn name(&self) -> String {
        "Arguments".to_string()
    }
    //
    // fn to_string(&self, _: &mut Realm) -> Result<YSString, Error> {
    //     Ok("[object Arguments]".into())
    // }
    //
    // fn to_string_internal(&self) -> Result<YSString, Error> {
    //     Ok("[object Arguments]".into())
    // }

    fn get_array_or_done(
        &self,
        index: usize,
        _: &mut Realm,
    ) -> Result<(bool, Option<Value>), Error> {
        let args = self.args.borrow();
        if index < args.len() {
            Ok((false, Some(args[index].clone())))
        } else {
            Ok((true, None))
        }
    }
}

#[props]
impl Arguments {
    #[prop(crate::Symbol::ITERATOR)]
    #[nonstatic]
    fn iterator(realm: &Realm, this: Value) -> ValueResult {
        let Value::Object(obj) = this else {
            return Err(crate::Error::ty_error(format!(
                "Expected object, found {this:?}"
            )));
        };

        let iter = ArrayIterator {
            inner: RefCell::new(MutableArrayIterator {
                object: MutObject::with_proto(realm.intrinsics.array_iter.clone()),
            }),
            array: obj,
            next: Cell::new(0),
            done: Cell::new(false),
        };

        let iter: Box<dyn Obj> = Box::new(iter);

        Ok(iter.into())
    }
}
