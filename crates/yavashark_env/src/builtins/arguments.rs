use crate::array::{ArrayIterator, MutableArrayIterator};
use crate::error::Error;
use crate::value::{Attributes, DefinePropertyResult, MutObj, Obj, ObjectImpl, Property};
use crate::{InternalPropertyKey, MutObject, PropertyKey, Realm, Res, Value, ValueResult, Variable};
use std::cell::{Cell, RefCell};
use std::ops::{Deref, DerefMut};
use yavashark_macro::props;

#[derive(Debug)]
pub struct Arguments {
    pub inner: RefCell<MutObject>,
    pub callee: Option<Value>,
    pub length: RefCell<Value>,
    pub args: RefCell<Vec<Value>>,
}

impl Arguments {
    pub fn new(args: Vec<Value>, callee: Option<Value>, realm: &mut Realm) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutObject::with_proto(realm.intrinsics.clone_public().arguments.get(realm)?.clone())),
            callee,
            length: RefCell::new(args.len().into()),
            args: RefCell::new(args),
        })
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

            if self.callee.is_none() && s == "callee" {
                return Err(Error::ty("Cannot redefine property: callee"));
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

            if self.callee.is_none() && s == "callee" {
                return Err(Error::ty("Cannot redefine property: callee"));
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
                let Some(callee) = &self.callee else {
                    return Ok(Some(Property::Getter(
                        realm.intrinsics.clone_public().throw_type_error.get(realm)?.clone(),
                        Attributes::from_values(false, false, false),
                    )))
                };


                return Ok(Some(Property::Value(
                    callee.clone(),
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
                let Some(callee) = &self.callee else {
                    return Ok(Some(Property::Getter(
                        realm.intrinsics.clone_public().throw_type_error.get(realm)?.clone(),
                        Attributes::from_values(false, false, false),
                    )))
                };


                return Ok(Some(Property::Value(
                    callee.clone(),
                    Attributes::write_config(),
                )));
            }
        }

        self.get_wrapped_object().get_own_property(name, realm)
    }

    fn contains_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool> {
        if let InternalPropertyKey::Index(idx) = name {
            if idx < self.args.borrow().len() {
                return Ok(true);
            }
        }

        if let InternalPropertyKey::String(s) = &name {
            if s == "length" {
                return Ok(true);
            }
            if s == "callee" {
                return Ok(true);
            }
        }

        self.get_wrapped_object().contains_key(name, realm)
    }

    fn contains_own_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool> {
        if let InternalPropertyKey::Index(idx) = name {
            if idx < self.args.borrow().len() {
                return Ok(true);
            }
        }

        if let InternalPropertyKey::String(s) = &name {
            if s == "length" {
                return Ok(true);
            }
            if s == "callee" {
                return Ok(true);
            }
        }

        self.get_wrapped_object().contains_own_key(name, realm)
    }

    fn properties(&self, realm: &mut Realm) -> Res<Vec<(PropertyKey, crate::value::Value)>> {
        let mut props = Vec::new();
        let args = self.args.borrow();
        for i in 0..args.len() {
            props.push((PropertyKey::from(i), args[i].clone()));
        }
        props.push((PropertyKey::from("length"), self.length.borrow().clone()));
        if let Some(callee) = &self.callee {
            props.push((PropertyKey::from("callee"), callee.clone()));
        }
        let mut parent_props = self.get_wrapped_object().properties(realm)?;
        props.append(&mut parent_props);
        Ok(props)
    }

    fn keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>> {
        let mut keys = Vec::new();
        let args = self.args.borrow();
        for i in 0..args.len() {
            keys.push(PropertyKey::from(i));
        }
        keys.push(PropertyKey::from("length"));
        if self.callee.is_some() {
            keys.push(PropertyKey::from("callee"));
        }
        let mut parent_keys = self.get_wrapped_object().keys(realm)?;
        keys.append(&mut parent_keys);
        Ok(keys)
    }

    fn values(&self, realm: &mut Realm) -> Res<Vec<crate::value::Value>> {
        let mut values = Vec::new();
        let args = self.args.borrow();
        for i in 0..args.len() {
            values.push(args[i].clone());
        }
        values.push(self.length.borrow().clone());
        if let Some(callee) = &self.callee {
            values.push(callee.clone());
        }
        let mut parent_values = self.get_wrapped_object().values(realm)?;
        values.append(&mut parent_values);
        Ok(values)
    }

    fn enumerable_properties(&self, realm: &mut Realm) -> Res<Vec<(PropertyKey, crate::value::Value)>> {
        let mut props = Vec::new();
        let args = self.args.borrow();
        for i in 0..args.len() {
            props.push((PropertyKey::from(i), args[i].clone()));
        }
        if let Some(callee) = &self.callee {
            props.push((PropertyKey::from("callee"), callee.clone()));
        }
        let mut parent_props = self.get_wrapped_object().enumerable_properties(realm)?;
        props.append(&mut parent_props);
        Ok(props)
    }

    fn enumerable_keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>> {
        let mut keys = Vec::new();
        let args = self.args.borrow();
        for i in 0..args.len() {
            keys.push(PropertyKey::from(i));
        }
        if let Some(_) = &self.callee {
            keys.push(PropertyKey::from("callee"));
        }
        let mut parent_keys = self.get_wrapped_object().enumerable_keys(realm)?;
        keys.append(&mut parent_keys);
        Ok(keys)
    }

    fn enumerable_values(&self, realm: &mut Realm) -> Res<Vec<crate::value::Value>> {
        let mut values = Vec::new();
        let args = self.args.borrow();
        for i in 0..args.len() {
            values.push(args[i].clone());
        }
        if let Some(callee) = &self.callee {
            values.push(callee.clone());
        }
        let mut parent_values = self.get_wrapped_object().enumerable_values(realm)?;
        values.append(&mut parent_values);
        Ok(values)
    }

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
    //
    // fn to_string(&self, _: &mut Realm) -> Result<YSString, Error> {
    //     Ok("[object Arguments]".into())
    // }
    //
    // fn to_string_internal(&self) -> Result<YSString, Error> {
    //     Ok("[object Arguments]".into())
    // }

    fn name(&self) -> String {
        "Arguments".to_string()
    }
}

#[props(intrinsic_name = arguments)]
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
