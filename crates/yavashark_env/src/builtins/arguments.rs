use crate::array::{ArrayIterator, ArrayIteratorKind, MutableArrayIterator};
use crate::error::Error;
use crate::value::{
    Attributes, DefinePropertyResult, MutObj, Obj, ObjectImpl, Property, PropertyDescriptor,
};
use crate::{
    InternalPropertyKey, MutObject, ObjectHandle, PropertyKey, Realm, Res, Value, ValueResult,
    Variable, scope::Scope,
};
use std::cell::{Cell, RefCell};
use std::ops::{Deref, DerefMut};
use yavashark_macro::props;

#[derive(Debug)]
pub struct Arguments {
    pub inner: RefCell<MutObject>,
    pub callee: Option<Value>,
    pub length: RefCell<Value>,
    pub args: RefCell<Vec<Value>>,
    pub unmapped: RefCell<Vec<usize>>,
    pub parameter_scope: Option<RefCell<Scope>>,
    pub parameter_names: Vec<Option<String>>,
}

impl Arguments {
    pub fn new(args: Vec<Value>, callee: Option<Value>, realm: &mut Realm) -> Res<Self> {
        Self::new_mapped(args, callee, None, Vec::new(), realm)
    }

    pub fn new_mapped(
        args: Vec<Value>,
        callee: Option<Value>,
        parameter_scope: Option<Scope>,
        parameter_names: Vec<Option<String>>,
        realm: &mut Realm,
    ) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutObject::with_proto(
                realm
                    .intrinsics
                    .clone_public()
                    .arguments
                    .get(realm)?
                    .clone(),
            )),
            callee,
            length: RefCell::new(args.len().into()),
            args: RefCell::new(args),
            unmapped: RefCell::new(Vec::new()),
            parameter_scope: parameter_scope.map(RefCell::new),
            parameter_names,
        })
    }

    fn is_mapped(&self, idx: usize) -> bool {
        idx < self.args.borrow().len() && self.unmapped.borrow().binary_search(&idx).is_err()
    }

    fn unmap_index(&self, idx: usize) {
        let mut unmapped = self.unmapped.borrow_mut();
        if let Err(pos) = unmapped.binary_search(&idx) {
            unmapped.insert(pos, idx);
        }
    }

    pub fn resolve_array(&self, idx: usize) -> Option<Value> {
        if self.is_mapped(idx) {
            Some(self.args.borrow().get(idx)?.copy())
        } else {
            None
        }
    }

    pub fn set_array(&self, idx: usize, value: Value) -> Res<()> {
        if self.is_mapped(idx) {
            if let Some(v) = self.args.borrow_mut().get_mut(idx) {
                *v = value;
                return Ok(());
            }
        }
        Err(Error::new("Index out of bounds or unmapped"))
    }

    fn update_parameter(&self, idx: usize, value: Value, realm: &mut Realm) -> Res {
        if let (Some(scope), Some(Some(name))) =
            (&self.parameter_scope, self.parameter_names.get(idx))
        {
            scope.borrow_mut().update(name, value, realm)?;
        }
        Ok(())
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
            if self.is_mapped(idx) {
                if let Some(v) = self.args.borrow_mut().get_mut(idx) {
                    *v = value.clone();
                    self.update_parameter(idx, value, realm)?;
                    return Ok(DefinePropertyResult::Handled);
                }
            }
            return self
                .get_wrapped_object()
                .define_property(name, value, realm);
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
            if self.is_mapped(idx) {
                if let Some(v) = self.args.borrow_mut().get_mut(idx) {
                    *v = value.value.clone();
                }
                self.update_parameter(idx, value.value.clone(), realm)?;
                let result = self.get_wrapped_object().define_property_attributes(
                    name,
                    value.clone(),
                    realm,
                )?;
                if !value.properties.is_writable() {
                    self.unmap_index(idx);
                }
                return Ok(result);
            }
            return self
                .get_wrapped_object()
                .define_property_attributes(name, value, realm);
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

    fn define_getter_attributes(
        &self,
        name: InternalPropertyKey,
        callback: ObjectHandle,
        attributes: Attributes,
        realm: &mut Realm,
    ) -> Res {
        if let InternalPropertyKey::Index(idx) = &name {
            if self.is_mapped(*idx) {
                self.unmap_index(*idx);
            }
        }
        self.get_wrapped_object()
            .define_getter_attributes(name, callback, attributes, realm)
    }

    fn define_setter_attributes(
        &self,
        name: InternalPropertyKey,
        callback: ObjectHandle,
        attributes: Attributes,
        realm: &mut Realm,
    ) -> Res {
        if let InternalPropertyKey::Index(idx) = &name {
            if self.is_mapped(*idx) {
                self.unmap_index(*idx);
            }
        }
        self.get_wrapped_object()
            .define_setter_attributes(name, callback, attributes, realm)
    }

    fn define_empty_accessor(
        &self,
        name: InternalPropertyKey,
        attributes: Attributes,
        realm: &mut Realm,
    ) -> Res {
        if let InternalPropertyKey::Index(idx) = &name {
            if self.is_mapped(*idx) {
                self.unmap_index(*idx);
            }
        }
        self.get_wrapped_object()
            .define_empty_accessor(name, attributes, realm)
    }

    fn delete_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        if let InternalPropertyKey::Index(idx) = name {
            if self.is_mapped(idx) {
                if self
                    .get_wrapped_object()
                    .get_property_descriptor(name.clone(), realm)?
                    .is_some_and(|descriptor| match descriptor {
                        PropertyDescriptor::Data { configurable, .. }
                        | PropertyDescriptor::Accessor { configurable, .. } => !configurable,
                    })
                {
                    return Ok(None);
                }
                let old = self.resolve_array(idx).map(Property::from);
                self.unmap_index(idx);
                let _ = self.get_wrapped_object().delete_property(name, realm)?;
                return Ok(old);
            }
        }
        self.get_wrapped_object().delete_property(name, realm)
    }

    fn resolve_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        if let InternalPropertyKey::Index(idx) = name {
            if let Some(value) = self.resolve_array(idx) {
                let attributes = self
                    .get_wrapped_object()
                    .get_property_descriptor(name.clone(), realm)?
                    .map_or(Attributes::new(), |descriptor| match descriptor {
                        PropertyDescriptor::Data {
                            writable,
                            enumerable,
                            configurable,
                            ..
                        } => Attributes::from_values(writable, enumerable, configurable),
                        PropertyDescriptor::Accessor { .. } => Attributes::new(),
                    });
                return Ok(Some(Property::Value(value, attributes)));
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
                        realm
                            .intrinsics
                            .clone_public()
                            .throw_type_error
                            .get(realm)?
                            .clone(),
                        Attributes::from_values(false, false, false),
                    )));
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
                let attributes = self
                    .get_wrapped_object()
                    .get_property_descriptor(name.clone(), realm)?
                    .map_or(Attributes::new(), |descriptor| match descriptor {
                        PropertyDescriptor::Data {
                            writable,
                            enumerable,
                            configurable,
                            ..
                        } => Attributes::from_values(writable, enumerable, configurable),
                        PropertyDescriptor::Accessor { .. } => Attributes::new(),
                    });
                return Ok(Some(Property::Value(value, attributes)));
            }
        }

        if let InternalPropertyKey::String(s) = &name {
            if s == "length" {
                return Ok(Some(self.length.borrow().clone().into()));
            }
            if s == "callee" {
                let Some(callee) = &self.callee else {
                    return Ok(Some(Property::Getter(
                        realm
                            .intrinsics
                            .clone_public()
                            .throw_type_error
                            .get(realm)?
                            .clone(),
                        Attributes::from_values(false, false, false),
                    )));
                };

                return Ok(Some(Property::Value(
                    callee.clone(),
                    Attributes::write_config(),
                )));
            }
        }

        self.get_wrapped_object().get_own_property(name, realm)
    }

    fn contains_own_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool> {
        if let InternalPropertyKey::Index(idx) = name {
            if self.is_mapped(idx) {
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

    fn contains_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool> {
        if let InternalPropertyKey::Index(idx) = name {
            if self.is_mapped(idx) {
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

    fn properties(&self, realm: &mut Realm) -> Res<Vec<(PropertyKey, Property)>> {
        let mut props = Vec::new();
        let args = self.args.borrow();
        for i in 0..args.len() {
            props.push((PropertyKey::from(i), args[i].clone().into()));
        }
        props.push((
            PropertyKey::from("length"),
            self.length.borrow().clone().into(),
        ));
        if let Some(callee) = &self.callee {
            props.push((PropertyKey::from("callee"), callee.clone().into()));
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

    fn values(&self, realm: &mut Realm) -> Res<Vec<Property>> {
        let mut values = Vec::new();
        let args = self.args.borrow();
        for i in 0..args.len() {
            values.push(args[i].clone().into());
        }
        values.push(self.length.borrow().clone().into());
        if let Some(callee) = &self.callee {
            values.push(callee.clone().into());
        }
        let mut parent_values = self.get_wrapped_object().values(realm)?;
        values.append(&mut parent_values);
        Ok(values)
    }

    fn enumerable_properties(&self, realm: &mut Realm) -> Res<Vec<(PropertyKey, Property)>> {
        let mut props = Vec::new();
        let args = self.args.borrow();
        for i in 0..args.len() {
            props.push((PropertyKey::from(i), args[i].clone().into()));
        }
        if let Some(callee) = &self.callee {
            props.push((PropertyKey::from("callee"), callee.clone().into()));
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

    fn enumerable_values(&self, realm: &mut Realm) -> Res<Vec<Property>> {
        let mut values = Vec::new();
        let args = self.args.borrow();
        for i in 0..args.len() {
            values.push(args[i].clone().into());
        }
        if let Some(callee) = &self.callee {
            values.push(callee.clone().into());
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

    fn get_property_descriptor(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<PropertyDescriptor>> {
        if let InternalPropertyKey::Index(idx) = name {
            if let Some(value) = self.resolve_array(idx) {
                if let Some(PropertyDescriptor::Data {
                    writable,
                    enumerable,
                    configurable,
                    ..
                }) = self
                    .get_wrapped_object()
                    .get_property_descriptor(name.clone(), realm)?
                {
                    return Ok(Some(PropertyDescriptor::Data {
                        value,
                        writable,
                        enumerable,
                        configurable,
                    }));
                }
                return Ok(Some(PropertyDescriptor::Data {
                    value,
                    writable: true,
                    enumerable: true,
                    configurable: true,
                }));
            }
        }

        if let InternalPropertyKey::String(s) = &name {
            if s == "length" {
                return Ok(Some(PropertyDescriptor::Data {
                    value: self.length.borrow().clone(),
                    writable: true,
                    enumerable: false,
                    configurable: false,
                }));
            }
            if s == "callee" {
                let Some(callee) = &self.callee else {
                    return Ok(Some(PropertyDescriptor::Accessor {
                        get: Some(
                            realm
                                .intrinsics
                                .clone_public()
                                .throw_type_error
                                .get(realm)?
                                .clone(),
                        ),
                        set: None,
                        enumerable: false,
                        configurable: false,
                    }));
                };

                return Ok(Some(PropertyDescriptor::Data {
                    value: callee.clone(),
                    writable: true,
                    enumerable: false,
                    configurable: true,
                }));
            }
        }

        self.get_wrapped_object()
            .get_property_descriptor(name, realm)
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
    fn iterator(realm: &mut Realm, this: Value) -> ValueResult {
        let Value::Object(obj) = this else {
            return Err(crate::Error::ty_error(format!(
                "Expected object, found {this:?}"
            )));
        };

        let iter = ArrayIterator {
            inner: RefCell::new(MutableArrayIterator {
                object: MutObject::with_proto(
                    realm
                        .intrinsics
                        .clone_public()
                        .array_iter
                        .get(realm)?
                        .clone(),
                ),
            }),
            array: obj,
            next: Cell::new(0),
            done: Cell::new(false),
            kind: ArrayIteratorKind::Values,
        };

        let iter: Box<dyn Obj> = Box::new(iter);

        Ok(iter.into())
    }
}
