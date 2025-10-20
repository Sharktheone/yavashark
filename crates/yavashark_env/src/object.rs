use crate::partial_init::Initializer;
use crate::realm::Realm;
use crate::value::property_key::{InternalPropertyKey, PropertyKey};
use crate::value::{BoxedObj, DefinePropertyResult, MutObj, Obj, ObjectOrNull, Property};
use crate::{Error, ObjectHandle, ObjectProperty, ValueResult, Variable};
use crate::{Res, Value};
use indexmap::map::Entry;
use indexmap::IndexMap;
pub use prototype::*;
use rustc_hash::FxBuildHasher;
use std::cell::{Ref, RefCell, RefMut};
use std::fmt::Debug;
use std::mem;
use yavashark_garbage::GcRef;

pub mod array;
pub mod constructor;

mod prealloc;
mod properties;
pub mod prototype;

#[derive(Debug)]
pub struct Object {
    inner: RefCell<MutObject>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MutObject {
    pub properties: IndexMap<PropertyKey, usize, FxBuildHasher>,
    pub array: Vec<(usize, usize)>,
    pub values: Vec<ObjectProperty>,
    pub prototype: ObjectOrNull,
}

impl Object {
    #[must_use]
    #[allow(clippy::new_ret_no_self)]
    pub fn new(realm: &Realm) -> ObjectHandle {
        ObjectHandle::new(Self::raw(realm))
    }

    #[must_use]
    pub fn null() -> ObjectHandle {
        Self::with_proto(None)
    }

    #[must_use]
    pub fn with_proto(proto: impl Into<ObjectOrNull>) -> ObjectHandle {
        ObjectHandle::new(Self::raw_with_proto(proto))
    }

    #[must_use]
    pub fn raw(realm: &Realm) -> Self {
        Self {
            inner: RefCell::new(MutObject::new(realm)),
        }
    }

    #[must_use]
    pub fn raw_with_proto(proto: impl Into<ObjectOrNull>) -> Self {
        Self {
            inner: RefCell::new(MutObject::with_proto(proto)),
        }
    }

    pub fn from_values(
        values: Vec<(PropertyKey, Value)>,
        realm: &mut Realm,
    ) -> Result<ObjectHandle, Error> {
        Ok(ObjectHandle::new(Self::raw_from_values(values, realm)?))
    }

    pub fn raw_from_values(
        values: Vec<(PropertyKey, Value)>,
        realm: &mut Realm,
    ) -> Result<Self, Error> {
        Ok(Self {
            inner: RefCell::new(MutObject::from_values(values, realm)?),
        })
    }

    pub fn from_values_with_proto(
        values: Vec<(PropertyKey, Value)>,
        proto: impl Into<ObjectOrNull>,
    ) -> Result<ObjectHandle, Error> {
        Ok(ObjectHandle::new(Self::raw_from_values_with_proto(
            values, proto,
        )?))
    }

    pub fn raw_from_values_with_proto(
        values: Vec<(PropertyKey, Value)>,
        proto: impl Into<ObjectOrNull>,
    ) -> Result<Self, Error> {
        let mut object = MutObject::with_proto(proto);

        for (key, value) in values {
            object.internal_define_property_no_realm(key.into(), value)?;
        }

        Ok(Self {
            inner: RefCell::new(object),
        })
    }

    pub fn inner_mut(&self) -> Result<RefMut<'_, MutObject>, Error> {
        self.inner
            .try_borrow_mut()
            .map_err(|_| Error::new("Failed to borrow object mutably"))
    }

    pub fn inner(&self) -> Result<Ref<'_, MutObject>, Error> {
        self.inner
            .try_borrow()
            .map_err(|_| Error::new("Failed to borrow object"))
    }

    #[must_use]
    pub const fn from_mut(inner: MutObject) -> Self {
        Self {
            inner: RefCell::new(inner),
        }
    }
}

#[allow(unused)]
impl Obj for Object {
    fn define_property(
        &self,
        name: InternalPropertyKey,
        value: crate::value::Value,
        realm: &mut Realm,
    ) -> Res<DefinePropertyResult> {
        self.inner_mut()?.define_property(name, value, realm)
    }

    fn define_property_attributes(
        &self,
        name: InternalPropertyKey,
        value: crate::value::Variable,
        realm: &mut Realm,
    ) -> Res<DefinePropertyResult> {
        self.inner_mut()?
            .define_property_attributes(name, value, realm)
    }

    fn resolve_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        self.inner()?.resolve_property(name, realm)
    }

    fn get_own_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        self.inner()?.get_own_property(name, realm)
    }

    fn define_getter(
        &self,
        name: InternalPropertyKey,
        callback: ObjectHandle,
        realm: &mut Realm,
    ) -> Res {
        self.inner_mut()?.define_getter(name, callback, realm)
    }

    fn define_setter(
        &self,
        name: InternalPropertyKey,
        callback: ObjectHandle,
        realm: &mut Realm,
    ) -> Res {
        self.inner_mut()?.define_setter(name, callback, realm)
    }

    fn delete_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        self.inner_mut()?.delete_property(name, realm)
    }

    fn contains_own_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool> {
        self.inner_mut()?.contains_own_key(name, realm)
    }

    fn contains_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool> {
        self.inner_mut()?.contains_key(name, realm)
    }

    fn properties(&self, realm: &mut Realm) -> Res<Vec<(PropertyKey, crate::value::Value)>> {
        self.inner()?.properties(realm)
    }

    fn keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>> {
        self.inner()?.keys(realm)
    }

    fn values(&self, realm: &mut Realm) -> Res<Vec<crate::value::Value>> {
        self.inner()?.values(realm)
    }

    fn enumerable_properties(
        &self,
        realm: &mut Realm,
    ) -> Res<Vec<(PropertyKey, crate::value::Value)>> {
        self.inner()?.enumerable_properties(realm)
    }

    fn enumerable_keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>> {
        self.inner()?.enumerable_keys(realm)
    }

    fn enumerable_values(&self, realm: &mut Realm) -> Res<Vec<crate::value::Value>> {
        self.inner()?.enumerable_values(realm)
    }

    fn clear_properties(&self, realm: &mut Realm) -> Res {
        self.inner_mut()?.clear_properties(realm)
    }

    fn get_array_or_done(
        &self,
        idx: usize,
        realm: &mut Realm,
    ) -> Res<(bool, Option<crate::value::Value>)> {
        self.inner_mut()?.get_array_or_done(idx, realm)
    }

    fn prototype(&self, realm: &mut Realm) -> Res<ObjectOrNull> {
        self.inner()?.prototype(realm)
    }

    fn set_prototype(&self, prototype: ObjectOrNull, realm: &mut Realm) -> Res {
        self.inner_mut()?.set_prototype(prototype.into(), realm)
    }

    fn gc_refs(&self) -> Vec<GcRef<BoxedObj>> {
        self.inner.borrow().gc_refs()
    }

    // impl Obj for Object {
    //     fn define_property(&self, name: Value, value: Value) -> Result<(), Error> {
    //         self.inner_mut()?.define_property(name, value)
    //     }
    //
    //     fn define_variable(&self, name: Value, value: Variable) -> Result<(), Error> {
    //         self.inner_mut()?.define_variable(name, value)
    //     }
    //
    //     fn resolve_property(&self, name: &Value) -> Result<Option<ObjectProperty>, Error> {
    //         self.inner()?.resolve_property(name)
    //     }
    //
    //     fn get_property(&self, name: &Value) -> Result<Option<ObjectProperty>, Error> {
    //         self.inner()?.get_property(name)
    //     }
    //
    //     fn define_getter(&self, name: Value, value: Value) -> Result<(), Error> {
    //         self.inner_mut()?.define_getter(name, value)
    //     }
    //
    //     fn define_setter(&self, name: Value, value: Value) -> Result<(), Error> {
    //         self.inner_mut()?.define_setter(name, value)
    //     }
    //
    //     fn delete_property(&self, name: &Value) -> Result<Option<Value>, Error> {
    //         self.inner_mut()?.delete_property(name)
    //     }
    //
    //     fn contains_key(&self, name: &Value) -> Result<bool, Error> {
    //         self.inner()?.contains_key(name)
    //     }
    //
    //     fn name(&self) -> String {
    //         self.inner()
    //             .map_or_else(|_| "Object".to_string(), |i| i.name())
    //     }
    //
    //     fn to_string(&self, realm: &mut Realm) -> Result<YSString, Error> {
    //         self.inner()?.to_string(realm)
    //     }
    //
    //     fn to_string_internal(&self) -> Result<YSString, Error> {
    //         self.inner()?.to_string_internal()
    //     }
    //
    //     fn properties(&self) -> Result<Vec<(Value, Value)>, Error> {
    //         self.inner()?.properties()
    //     }
    //
    //     fn keys(&self) -> Result<Vec<Value>, Error> {
    //         self.inner()?.keys()
    //     }
    //
    //     fn values(&self) -> Result<Vec<Value>, Error> {
    //         self.inner()?.values()
    //     }
    //
    //     fn get_array_or_done(&self, index: usize) -> Result<(bool, Option<Value>), Error> {
    //         self.inner()?.get_array_or_done(index)
    //     }
    //
    //     fn clear_values(&self) -> Result<(), Error> {
    //         self.inner_mut()?.clear_values()
    //     }
    //
    //     fn call(&self, realm: &mut Realm, args: Vec<Value>, this: Value) -> Result<Value, Error> {
    //         self.inner_mut()?.call(realm, args, this)
    //     }
    //
    //     fn prototype(&self) -> Result<ObjectProperty, Error> {
    //         self.inner()?.prototype()
    //     }
    //
    //     fn constructor(&self) -> Result<ObjectProperty, Error> {
    //         self.inner()?.constructor()
    //     }
    //
    //     unsafe fn custom_gc_refs(&self) -> Vec<GcRef<BoxedObj>> {
    //         self.inner().map(|o| o.custom_gc_refs()).unwrap_or_default()
    //     }
}

impl MutObject {
    #[must_use]
    pub fn new(realm: &Realm) -> Self {
        let prototype = realm.intrinsics.obj.clone().into();

        Self {
            properties: Default::default(),
            array: Vec::new(),
            values: Vec::new(),
            prototype,
        }
    }

    #[must_use]
    pub fn null() -> Self {
        Self {
            properties: Default::default(),
            prototype: ObjectOrNull::Null,
            values: Vec::new(),
            array: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_proto(proto: impl Into<ObjectOrNull>) -> Self {
        Self {
            properties: Default::default(),
            prototype: proto.into(),
            values: Vec::new(),
            array: Vec::new(),
        }
    }

    #[must_use]
    pub fn array_position(&self, index: usize) -> (usize, bool) {
        if self.array.is_empty() {
            return (0, false);
        }

        if self.array.len() > 100 {
            return self
                .array
                .binary_search_by(|(i, _)| i.cmp(&index))
                .map_or_else(|i| (i, false), |i| (i, true));
        }

        for (i, (j, _)) in self.array.iter().enumerate() {
            if *j == index {
                return (i, true);
            }

            if *j > index {
                return (i, false);
            }
        }

        (self.array.len(), false)
    }

    pub fn insert_array(&mut self, index: usize, value: impl Into<ObjectProperty>) {
        let (i, found) = self.array_position(index);

        if found {
            if let Some(vi) = self.array.get(i) {
                let Some(v) = self.values.get_mut(vi.1) else {
                    return;
                };

                if v.attributes.is_writable() {
                    *v = value.into();
                }
                return;
            }
        }

        let property_key = InternalPropertyKey::Index(index).into();

        if let Some(prop) = self.properties.get(&property_key) {
            let Some(v) = self.values.get_mut(*prop) else {
                return;
            };

            if v.attributes.is_writable() {
                *v = value.into();
                return;
            }
        }

        let len = self.values.len();
        self.values.push(value.into());

        self.array.insert(i, (index, len));
        self.properties.insert(property_key, len);
    }

    #[must_use]
    pub fn resolve_array(&self, index: usize) -> Option<Property> {
        let (i, found) = self.array_position(index);

        if found {
            return self
                .array
                .get(i)
                .and_then(|v| self.values.get(v.1).map(|p| p.property()));
        }

        None
    }

    #[must_use]
    pub fn get_array(&self, index: usize) -> Option<Property> {
        let (i, found) = self.array_position(index);

        if found {
            return self
                .array
                .get(i)
                .and_then(|v| self.values.get(v.1).map(|p| p.property()));
        }

        None
    }

    pub fn delete_array(&mut self, index: usize) -> Option<Property> {
        let (i, found) = self.array_position(index);

        if found {
            if !self.array.get(i).is_some_and(|v| {
                let Some(v) = self.values.get(v.1) else {
                    return false;
                };

                v.attributes.is_configurable()
            }) {
                return None;
            }

            let idx = self.array.remove(i);

            return self
                .values
                .get_mut(idx.1)
                .map(|v| mem::replace(v, ObjectProperty::new(Value::Undefined)).property());
        }

        Some(Property::default())
    }

    pub fn set_array(&mut self, elements: impl ExactSizeIterator<Item = Value>) {
        self.array.clear();

        let len = self.values.len();
        let elements_len = elements.len();

        self.values.extend(elements.map(ObjectProperty::new));

        for i in 0..elements_len {
            self.array.push((i, len + i));
        }
    }

    pub fn set_array_res(&mut self, elements: impl ExactSizeIterator<Item = ValueResult>) -> Res {
        self.array.clear();

        let len = self.values.len();
        let elements_len = elements.len();

        for val in elements {
            let val = val?;
            self.values.push(ObjectProperty::new(val));
        }

        for i in 0..elements_len {
            self.array.push((i, len + i));
        }

        Ok(())
    }

    pub fn get_array_mut(&mut self, index: usize) -> Option<&mut Value> {
        let (i, found) = self.array_position(index);

        if found {
            return self
                .array
                .get(i)
                .and_then(|v| self.values.get_mut(v.1).map(|p| &mut p.value)); //TODO: Check for perms
        }

        None
    }

    #[must_use]
    pub fn contains_array_key(&self, index: usize) -> bool {
        let (_, found) = self.array_position(index);

        found
    }

    pub fn resize_array(&mut self, new_len: usize) {
        let len = self.array.last().map_or(0, |v| v.0 + 1);

        if len > new_len {
            let (idx, _) = self.array_position(new_len);

            self.array.truncate(idx);
        }
    }

    pub fn from_values(
        values: Vec<(PropertyKey, Value)>,
        realm: &mut Realm,
    ) -> Result<Self, Error> {
        let mut object = Self::new(realm);

        for (key, value) in values {
            object.define_property(key.into(), value, realm)?;
        }

        Ok(object)
    }

    pub fn force_update_property_cb(
        &mut self,
        key: PropertyKey,
        cb: impl FnOnce(Option<&mut ObjectProperty>) -> Option<Value>,
    ) -> Res {
        match self.properties.entry(key) {
            Entry::Occupied(entry) => {
                let idx = *entry.get();

                let Some(val) = cb(self.values.get_mut(idx)) else {
                    return Ok(());
                };

                if let Some(v) = self.values.get_mut(idx) {
                    v.value = val;
                } else {
                    return Err(Error::new("Failed to get value for property"));
                }
            }
            Entry::Vacant(entry) => {
                let Some(val) = cb(None) else {
                    return Ok(());
                };

                let idx = self.values.len();
                self.values.push(ObjectProperty::new(val));
                entry.insert(idx);
            }
        }

        Ok(())
    }

    fn internal_define_property_no_realm(
        &mut self,
        name: InternalPropertyKey,
        value: Value,
    ) -> Res<DefinePropertyResult> {
        if let InternalPropertyKey::Index(n) = name {
            self.insert_array(n, value);
            return Ok(DefinePropertyResult::Handled);
        }

        if let InternalPropertyKey::String(s) = &name {
            if s == "__proto__" {
                self.prototype = ObjectOrNull::try_from(value)?;
                return Ok(DefinePropertyResult::Handled);
            }
        }

        match self.properties.entry(name.into()) {
            Entry::Occupied(entry) => {
                let Some(e) = self.values.get_mut(*entry.get()) else {
                    return Err(Error::new("Failed to get value for property"));
                };

                if e.attributes.is_writable() {
                    e.value = value;
                    return Ok(DefinePropertyResult::Handled);
                }
            }
            Entry::Vacant(entry) => {
                let idx = self.values.len();
                self.values.push(ObjectProperty::new(value));
                entry.insert(idx);
            }
        }
        Ok(DefinePropertyResult::Handled)
    }
}

impl MutObj for MutObject {
    fn define_property(
        &mut self,
        name: InternalPropertyKey,
        value: Value,
        _realm: &mut Realm,
    ) -> Res<DefinePropertyResult> {
        if let InternalPropertyKey::Index(n) = name {
            self.insert_array(n, value);
            return Ok(DefinePropertyResult::Handled);
        }

        if let InternalPropertyKey::String(s) = &name {
            if s == "__proto__" {
                self.prototype = ObjectOrNull::try_from(value)?;
                return Ok(DefinePropertyResult::Handled);
            }
        }

        match self.properties.entry(name.into()) {
            Entry::Occupied(entry) => {
                let Some(e) = self.values.get_mut(*entry.get()) else {
                    return Err(Error::new("Failed to get value for property"));
                };

                if !e.set.is_undefined() {
                    return Ok(DefinePropertyResult::Setter(
                        e.set.as_object()?.clone(),
                        value,
                    ));
                }

                if !e.get.is_undefined() {
                    return Ok(DefinePropertyResult::ReadOnly);
                }

                return Ok(if e.attributes.is_writable() {
                    e.set = Value::Undefined;
                    e.get = Value::Undefined;

                    e.value = value;
                    DefinePropertyResult::Handled
                } else {
                    DefinePropertyResult::ReadOnly
                });
            }
            Entry::Vacant(entry) => {
                let idx = self.values.len();
                self.values.push(ObjectProperty::new(value));
                entry.insert(idx);
            }
        }
        Ok(DefinePropertyResult::Handled)
    }

    fn define_property_attributes(
        &mut self,
        name: InternalPropertyKey,
        value: Variable,
        _realm: &mut Realm,
    ) -> Res<DefinePropertyResult> {
        if let InternalPropertyKey::Index(n) = name {
            self.insert_array(n, value);
            return Ok(DefinePropertyResult::Handled);
        }

        if let InternalPropertyKey::String(s) = &name {
            if s == "__proto__" {
                self.prototype = ObjectOrNull::try_from(value.value)?;
                return Ok(DefinePropertyResult::Handled);
            }
        }

        match self.properties.entry(name.into()) {
            Entry::Occupied(entry) => {
                let Some(e) = self.values.get_mut(*entry.get()) else {
                    return Err(Error::new("Failed to get value for property"));
                };

                if !e.set.is_undefined() {
                    return Ok(DefinePropertyResult::Setter(
                        e.set.as_object()?.clone(),
                        value.value,
                    ));
                }

                if !e.get.is_undefined() {
                    return Ok(DefinePropertyResult::ReadOnly);
                }

                return Ok(if e.attributes.is_writable() {
                    *e = value.into();

                    DefinePropertyResult::Handled
                } else {
                    DefinePropertyResult::ReadOnly
                });
            }
            Entry::Vacant(entry) => {
                let idx = self.values.len();
                self.values.push(value.into());
                entry.insert(idx);
            }
        }

        Ok(DefinePropertyResult::Handled)
    }

    fn resolve_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        if matches!(&name, InternalPropertyKey::String(str) if str == "__proto__") {
            let val: Value = self.prototype.clone().into();
            return Ok(Some(val.into()));
        }

        if let InternalPropertyKey::Index(n) = name {
            if let Some(value) = self.resolve_array(n) {
                return Ok(Some(value));
            }
            //TODO: we should insert a new reference in the array to the value if we find it in the property map
        }

        if let Some(prop) = self
            .properties
            .get::<PropertyKey>(&name.clone().into())
            .and_then(|idx| self.values.get(*idx).map(ObjectProperty::property))
        {
            return Ok(Some(prop));
        }

        match &self.prototype {
            ObjectOrNull::Object(o) => {
                o.resolve_property_no_get_set(name, realm)
                //TODO: this is wrong, we need a realm here!
            }
            _ => Ok(None),
        }
    }

    fn get_own_property(
        &self,
        name: InternalPropertyKey,
        _realm: &mut Realm,
    ) -> Res<Option<Property>> {
        if matches!(&name, InternalPropertyKey::String(str) if str == "__proto__") {
            let val: Value = self.prototype.clone().into();
            return Ok(Some(val.into()));
        }

        if let InternalPropertyKey::Index(n) = name {
            return Ok(self.get_array(n));
        }

        if let Some(prop) = self.properties.get::<PropertyKey>(&name.into()) {
            return Ok(self.values.get(*prop).map(|v| v.property()));
        }

        Ok(None)
    }

    fn define_getter(
        &mut self,
        name: InternalPropertyKey,
        value: ObjectHandle,
        _realm: &mut Realm,
    ) -> Res {
        if let InternalPropertyKey::Index(n) = name {
            self.insert_array(n, ObjectProperty::getter(value.into()));
            return Ok(());
        }

        let key = name.into();

        let val = self
            .properties
            .get_mut(&key)
            .and_then(|idx| self.values.get_mut(*idx));

        if let Some(prop) = val {
            prop.get = value.into();
            return Ok(());
        }

        match self.properties.entry(key) {
            Entry::Occupied(entry) => {
                let Some(e) = self.values.get_mut(*entry.get()) else {
                    return Err(Error::new("Failed to get value for property"));
                };

                e.value = Value::Undefined;
                e.get = value.into();
                return Ok(());
            }
            Entry::Vacant(entry) => {
                let idx = self.values.len();
                self.values.push(ObjectProperty::getter(value.into()));
                entry.insert(idx);
            }
        }

        Ok(())
    }

    fn define_setter(
        &mut self,
        name: InternalPropertyKey,
        value: ObjectHandle,
        _realm: &mut Realm,
    ) -> Res {
        if let InternalPropertyKey::Index(n) = name {
            self.insert_array(n, ObjectProperty::setter(value.into()));
            return Ok(());
        }

        let key = name.into();

        let val = self
            .properties
            .get_mut(&key)
            .and_then(|idx| self.values.get_mut(*idx));

        if let Some(prop) = val {
            prop.set = value.into();
            return Ok(());
        }

        match self.properties.entry(key) {
            Entry::Occupied(entry) => {
                let Some(e) = self.values.get_mut(*entry.get()) else {
                    return Err(Error::new("Failed to get value for property"));
                };

                e.value = Value::Undefined;
                e.set = value.into();
                return Ok(());
            }
            Entry::Vacant(entry) => {
                let idx = self.values.len();
                self.values.push(ObjectProperty::setter(value.into()));
                entry.insert(idx);
            }
        }

        Ok(())
    }

    fn delete_property(
        &mut self,
        name: InternalPropertyKey,
        _realm: &mut Realm,
    ) -> Res<Option<Property>> {
        if matches!(&name, InternalPropertyKey::String(str) if str == "__proto__") {
            return Ok(None);
        }

        if let InternalPropertyKey::Index(n) = name {
            return Ok(self.delete_array(n));
        }

        if let Entry::Occupied(occ) = self.properties.entry(name.into()) {
            let prop = self
                .values
                .get_mut(*occ.get())
                .ok_or_else(|| Error::new("Failed to get value for property"))?;

            return if prop.attributes.is_configurable() {
                occ.shift_remove();
                Ok(Some(mem::replace(prop, Value::Undefined.into()).property()))
            } else {
                // Err(Error::ty("Property is not configurable")) // this is only in strict mode
                Ok(None)
            };
        }

        Ok(Some(Property::default()))
    }

    fn contains_own_key(&mut self, name: InternalPropertyKey, _realm: &mut Realm) -> Res<bool> {
        if matches!(&name, InternalPropertyKey::String(str) if str == "__proto__") {
            return Ok(true);
        }

        if let InternalPropertyKey::Index(n) = name {
            return Ok(self.contains_array_key(n));
        }

        Ok(self.properties.contains_key::<PropertyKey>(&name.into()))
    }

    fn contains_key(
        &mut self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Result<bool, Error> {
        if matches!(&name, InternalPropertyKey::String(str) if str == "__proto__") {
            return Ok(true);
        }

        if let InternalPropertyKey::Index(n) = name {
            return Ok(self.contains_array_key(n));
        }

        if self
            .properties
            .contains_key::<PropertyKey>(&name.clone().into())
        {
            return Ok(true);
        }

        if let ObjectOrNull::Object(obj) = &self.prototype {
            return obj.contains_key(name, realm);
        }

        Ok(false)
    }

    // fn name(&self) -> String {
    //     "Object".to_string()
    // }

    // fn to_string(&self, _realm: &mut Realm) -> Result<YSString, Error> {
    //     Ok("[object Object]".into())
    // }
    //
    // fn to_string_internal(&self) -> Result<YSString, Error> {
    //     Ok("[object Object]".into())
    // }

    fn properties(&self, _realm: &mut Realm) -> Res<Vec<(PropertyKey, Value)>> {
        Ok(self
            .properties
            .iter()
            .filter_map(|(k, v)| {
                let v = self.values.get(*v)?;

                Some((k.clone(), v.value.copy()))
            })
            .collect())
    }

    fn keys(&self, _realm: &mut Realm) -> Res<Vec<PropertyKey>> {
        Ok(self
            .array
            .iter()
            .map(|(i, _)| InternalPropertyKey::Index(*i).into())
            .chain(self.properties.keys().cloned().map(Into::into))
            .collect::<Vec<_>>())
    }

    fn values(&self, _realm: &mut Realm) -> Res<Vec<Value>, Error> {
        Ok(self.values.iter().map(|v| v.value.copy()).collect())
        //TODO: getter (and setter) values
    }

    fn enumerable_properties(
        &self,
        _realm: &mut Realm,
    ) -> Res<Vec<(PropertyKey, crate::value::Value)>> {
        Ok(self
            .properties
            .iter()
            .filter_map(|(k, v)| {
                let v = self.values.get(*v)?;

                if v.attributes.is_enumerable() {
                    Some((k.clone().into(), v.value.copy()))
                } else {
                    None
                }
            })
            .collect())
    }

    fn enumerable_keys(&self, _realm: &mut Realm) -> Res<Vec<PropertyKey>> {
        Ok(self
            .array
            .iter()
            .map(|(i, _)| InternalPropertyKey::Index(*i).into())
            .chain(self.properties.iter().filter_map(|(k, v)| {
                let v = self.values.get(*v)?;

                if v.attributes.is_enumerable() {
                    Some(k.clone().into())
                } else {
                    None
                }
            }))
            .collect::<Vec<_>>())
    }

    fn enumerable_values(&self, _realm: &mut Realm) -> Res<Vec<crate::value::Value>> {
        Ok(self
            .values
            .iter()
            .filter_map(|v| {
                if v.attributes.is_enumerable() {
                    Some(v.value.copy())
                } else {
                    None
                }
            })
            .collect())
    }

    fn clear_properties(&mut self, _realm: &mut Realm) -> Res {
        self.properties.clear();
        self.array.clear();
        self.values.clear();

        Ok(())
    }

    fn get_array_or_done(&mut self, index: usize, realm: &mut Realm) -> Res<(bool, Option<Value>)> {
        if let Some(value) = self.resolve_array(index) {
            let done = if let Some((i, _)) = self.array.last() {
                index > *i
            } else {
                true
            };
            return Ok((done, Some(value.assert_value().value)));
        }

        if let ObjectOrNull::Object(obj) = &self.prototype {
            return obj.get_array_or_done(index, realm);
        }

        Ok((true, None))
    }

    // fn clear_values(&mut self) -> Res {
    //     self.properties.clear();
    //     self.array.clear();
    //
    //     Ok(())
    // }

    fn prototype(&self, _realm: &mut Realm) -> Result<ObjectOrNull, Error> {
        Ok(self.prototype.clone())
    }

    fn set_prototype(&mut self, proto: ObjectOrNull, _realm: &mut Realm) -> Res {
        self.prototype = proto;
        Ok(())
    }

    fn gc_refs(&self) -> Vec<GcRef<BoxedObj>> {
        let mut refs = Vec::new();

        if let ObjectOrNull::Object(o) = &self.prototype {
            if let Some(o) = o.gc_ref() {
                refs.push(o);
            }
        }

        for value in &self.values {
            if let Some(getter) = value.get.gc_ref() {
                refs.push(getter);
            }

            if let Some(setter) = value.set.gc_ref() {
                refs.push(setter);
            }

            if let Some(o) = value.value.gc_ref() {
                refs.push(o);
            }
        }

        refs
    }

    // fn constructor(&self) -> Result<ObjectProperty, Error> {
    //     if let Some(constructor) = self
    //         .properties
    //         .get(&PropertyKey::String("constructor".into()))
    //     {
    //         if let Some(value) = self.values.get(*constructor) {
    //             return Ok(value.clone());
    //         }
    //     }
    //
    //     if let Value::Object(proto) = self.prototype()?.value {
    //         let proto = proto.guard();
    //
    //         return proto.constructor();
    //     }
    //
    //     Ok(Value::Undefined.into())
    // }
}

// #[allow(unused)]
// impl MutObj for MutObject {
//     fn define_property(&mut self, name: InternalPropertyKey, value: crate::value::Value, realm: &mut Realm) -> Res<DefinePropertyResult> {
//         todo!()
//     }
//
//     fn define_property_attributes(&mut self, name: InternalPropertyKey, value: crate::value::Variable, realm: &mut Realm) -> Res<DefinePropertyResult> {
//         todo!()
//     }
//
//     fn resolve_property(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<Option<Property>> {
//         todo!()
//     }
//
//     fn get_own_property(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<Option<Property>> {
//         todo!()
//     }
//
//     fn define_getter(&mut self, name: InternalPropertyKey, callback: ObjectHandle, realm: &mut Realm) -> Res {
//         todo!()
//     }
//
//     fn define_setter(&mut self, name: InternalPropertyKey, callback: ObjectHandle, realm: &mut Realm) -> Res {
//         todo!()
//     }
//
//     fn delete_property(&mut self, name: InternalPropertyKey, realm: &mut Realm) -> Res<Option<Property>> {
//         todo!()
//     }
//
//     fn contains_own_key(&mut self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool> {
//         todo!()
//     }
//
//     fn contains_key(&mut self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool> {
//         todo!()
//     }
//
//     fn properties(&self, realm: &mut Realm) -> Res<Vec<(PropertyKey, crate::value::Value)>> {
//         todo!()
//     }
//
//     fn keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>> {
//         todo!()
//     }
//
//     fn values(&self, realm: &mut Realm) -> Res<Vec<crate::value::Value>> {
//         todo!()
//     }
//
//     fn enumerable_properties(&self, realm: &mut Realm) -> Res<Vec<(PropertyKey, crate::value::Value)>> {
//         todo!()
//     }
//
//     fn enumerable_keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>> {
//         todo!()
//     }
//
//     fn enumerable_values(&self, realm: &mut Realm) -> Res<Vec<crate::value::Value>> {
//         todo!()
//     }
//
//     fn clear_properties(&mut self, realm: &mut Realm) -> Res {
//         todo!()
//     }
//
//     fn get_array_or_done(&mut self, idx: usize, realm: &mut Realm) -> Res<(bool, Option<crate::value::Value>)> {
//         todo!()
//     }
//
//     fn prototype(&self, realm: &mut Realm) -> Res<ObjectOrNull> {
//         todo!()
//     }
//
//     fn set_prototype(&mut self, prototype: ObjectOrNull, realm: &mut Realm) -> Res {
//         todo!()
//     }
//
//     fn gc_refs(&self) -> Vec<GcRef<BoxedObj>> {
//         todo!()
//     }
// }

// #[cfg(test)]
// mod tests {
//     use crate::Value;
//
//     use super::*;
//
//     #[test]
//     fn object_creation_with_proto() {
//         let proto = Value::Null;
//         let object = Object::with_proto(None);
//
//         assert_eq!(
//             object.get_property(&"__proto__".into()).unwrap().value,
//             proto
//         );
//     }
//
//     #[test]
//     fn object_creation_raw_with_proto() {
//         let realm = Realm::new().unwrap();
//         let object = Object::raw(&realm);
//
//         let proto = object.into_object();
//
//         let object = Object::raw_with_proto(proto.clone());
//
//         assert_eq!(object.prototype().unwrap().value, Value::Object(proto));
//     }
//
//     #[test]
//     fn array_position_empty_array() {
//         let realm = Realm::new().unwrap();
//         let object = Object::raw(&realm);
//
//         let (index, found) = object.inner().unwrap().array_position(0);
//
//         assert_eq!(index, 0);
//         assert!(!found);
//     }
//
//     #[test]
//     fn array_position_non_empty_array() {
//         let realm = Realm::new().unwrap();
//         let object = Object::raw(&realm);
//         object
//             .inner_mut()
//             .unwrap()
//             .insert_array(0, Value::Number(42.0));
//
//         let (index, found) = object.inner().unwrap().array_position(0);
//
//         assert_eq!(index, 0);
//         assert!(found);
//     }
//
//     #[test]
//     fn insert_array() {
//         let realm = Realm::new().unwrap();
//         let object = Object::raw(&realm);
//         object
//             .inner_mut()
//             .unwrap()
//             .insert_array(0, Value::Number(42.0));
//
//         let inner = object.inner().unwrap();
//         let array_index = inner.array[0].1;
//         assert_eq!(inner.values[array_index].value, Value::Number(42.0));
//     }
//
//     #[test]
//     fn resolve_array() {
//         let realm = Realm::new().unwrap();
//         let object = Object::raw(&realm);
//         object
//             .inner_mut()
//             .unwrap()
//             .insert_array(0, Value::Number(42.0));
//
//         let value = object.inner().unwrap().resolve_array(0);
//
//         assert_eq!(value, Some(Value::Number(42.0).into()));
//     }
//
//     #[test]
//     fn get_array() {
//         let realm = Realm::new().unwrap();
//         let object = Object::raw(&realm);
//         object
//             .inner_mut()
//             .unwrap()
//             .insert_array(0, Value::Number(42.0));
//
//         let inner = object.inner().unwrap();
//
//         let value = inner.get_array(0).unwrap();
//
//         assert_eq!(&value.value, &Value::Number(42.0));
//     }
//
//     #[test]
//     fn get_array_mut() {
//         let realm = Realm::new().unwrap();
//         let object = Object::raw(&realm);
//         object
//             .inner_mut()
//             .unwrap()
//             .insert_array(0, Value::Number(42.0));
//
//         let mut inner = object.inner_mut().unwrap();
//
//         let value = inner.get_array_mut(0).unwrap();
//
//         assert_eq!(value, &mut Value::Number(42.0));
//     }
//
//     #[test]
//     fn contains_array_key() {
//         let realm = Realm::new().unwrap();
//         let object = Object::raw(&realm);
//         object
//             .inner_mut()
//             .unwrap()
//             .insert_array(0, Value::Number(42.0));
//
//         let contains = object.inner().unwrap().contains_array_key(0);
//
//         assert!(contains);
//     }
//
//     #[test]
//     fn define_property() {
//         let realm = Realm::new().unwrap();
//         let object = Object::raw(&realm);
//         object
//             .define_property(Value::string("key"), Value::Number(42.0))
//             .unwrap();
//
//         let inner = object.inner().unwrap();
//         let key: PropertyKey = Value::string("key").into();
//         let property_index = inner.properties.get(&key).unwrap();
//         assert_eq!(inner.values[*property_index].value, Value::Number(42.0));
//     }
//
//     #[test]
//     fn resolve_property() {
//         let realm = Realm::new().unwrap();
//         let object = Object::raw(&realm);
//         object
//             .define_property(Value::string("key"), Value::Number(42.0))
//             .unwrap();
//
//         let property = object
//             .resolve_property(&Value::string("key"))
//             .unwrap()
//             .unwrap();
//
//         assert_eq!(property.value, Value::Number(42.0));
//     }
//
//     #[test]
//     fn contains_key() {
//         let realm = Realm::new().unwrap();
//         let object = Object::raw(&realm);
//         object
//             .define_property(Value::string("key"), Value::Number(42.0))
//             .unwrap();
//
//         let contains = object.contains_key(&Value::string("key")).unwrap();
//
//         assert!(contains);
//     }
// }
