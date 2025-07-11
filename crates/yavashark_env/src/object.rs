use std::collections::hash_map::Entry;
pub use prototype::*;
use rustc_hash::FxHashMap;
use std::cell::{Ref, RefCell, RefMut};
use std::fmt::Debug;
use std::mem;
use yavashark_garbage::GcRef;
use yavashark_string::YSString;
use yavashark_value::{BoxedObj, MutObj, Obj};
use yavashark_value::property_key::{InternalPropertyKey, PropertyKey};
use crate::realm::Realm;
use crate::{Error, ObjectHandle, ObjectProperty, Variable};
use crate::{Res, Value};

pub mod array;
pub mod constructor;

mod prealloc;
pub mod prototype;

#[derive(Debug)]
pub struct Object {
    inner: RefCell<MutObject>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MutObject {
    pub properties: FxHashMap<PropertyKey, usize>,
    pub array: Vec<(usize, usize)>,
    pub values: Vec<ObjectProperty>,
    pub prototype: ObjectProperty,
}

impl Object {
    #[must_use]
    #[allow(clippy::new_ret_no_self)]
    pub fn new(realm: &Realm) -> ObjectHandle {
        ObjectHandle::new(Self::raw(realm))
    }

    #[must_use]
    pub fn null() -> ObjectHandle {
        Self::with_proto(Value::Null)
    }

    #[must_use]
    pub fn with_proto(proto: Value) -> ObjectHandle {
        ObjectHandle::new(Self::raw_with_proto(proto))
    }

    #[must_use]
    pub fn raw(realm: &Realm) -> Self {
        Self {
            inner: RefCell::new(MutObject::new(realm)),
        }
    }

    #[must_use]
    pub fn raw_with_proto(proto: Value) -> Self {
        Self {
            inner: RefCell::new(MutObject::with_proto(proto)),
        }
    }

    pub fn from_values(values: Vec<(Value, Value)>, realm: &Realm) -> Result<ObjectHandle, Error> {
        Ok(ObjectHandle::new(Self::raw_from_values(values, realm)?))
    }

    pub fn raw_from_values(values: Vec<(Value, Value)>, realm: &Realm) -> Result<Self, Error> {
        Ok(Self {
            inner: RefCell::new(MutObject::from_values(values, realm)?),
        })
    }

    pub fn inner_mut(&self) -> Result<RefMut<MutObject>, Error> {
        self.inner
            .try_borrow_mut()
            .map_err(|_| Error::new("Failed to borrow object mutably"))
    }

    pub fn inner(&self) -> Result<Ref<MutObject>, Error> {
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

impl Obj<Realm> for Object {
    fn define_property(&self, name: Value, value: Value) -> Result<(), Error> {
        self.inner_mut()?.define_property(name, value)
    }

    fn define_variable(&self, name: Value, value: Variable) -> Result<(), Error> {
        self.inner_mut()?.define_variable(name, value)
    }

    fn resolve_property(&self, name: &Value) -> Result<Option<ObjectProperty>, Error> {
        self.inner()?.resolve_property(name)
    }

    fn get_property(&self, name: &Value) -> Result<Option<ObjectProperty>, Error> {
        self.inner()?.get_property(name)
    }

    fn define_getter(&self, name: Value, value: Value) -> Result<(), Error> {
        self.inner_mut()?.define_getter(name, value)
    }

    fn define_setter(&self, name: Value, value: Value) -> Result<(), Error> {
        self.inner_mut()?.define_setter(name, value)
    }

    fn delete_property(&self, name: &Value) -> Result<Option<Value>, Error> {
        self.inner_mut()?.delete_property(name)
    }

    fn contains_key(&self, name: &Value) -> Result<bool, Error> {
        self.inner()?.contains_key(name)
    }

    fn name(&self) -> String {
        self.inner()
            .map_or_else(|_| "Object".to_string(), |i| i.name())
    }

    fn to_string(&self, realm: &mut Realm) -> Result<YSString, Error> {
        self.inner()?.to_string(realm)
    }

    fn to_string_internal(&self) -> Result<YSString, Error> {
        self.inner()?.to_string_internal()
    }

    fn properties(&self) -> Result<Vec<(Value, Value)>, Error> {
        self.inner()?.properties()
    }

    fn keys(&self) -> Result<Vec<Value>, Error> {
        self.inner()?.keys()
    }

    fn values(&self) -> Result<Vec<Value>, Error> {
        self.inner()?.values()
    }

    fn get_array_or_done(&self, index: usize) -> Result<(bool, Option<Value>), Error> {
        self.inner()?.get_array_or_done(index)
    }

    fn clear_values(&self) -> Result<(), Error> {
        self.inner_mut()?.clear_values()
    }

    fn call(&self, realm: &mut Realm, args: Vec<Value>, this: Value) -> Result<Value, Error> {
        self.inner_mut()?.call(realm, args, this)
    }

    fn prototype(&self) -> Result<ObjectProperty, Error> {
        self.inner()?.prototype()
    }

    fn constructor(&self) -> Result<ObjectProperty, Error> {
        self.inner()?.constructor()
    }

    unsafe fn custom_gc_refs(&self) -> Vec<GcRef<BoxedObj<Realm>>> {
        self.inner().map(|o| o.custom_gc_refs()).unwrap_or_default()
    }
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
            prototype: Value::Null.into(),
            values: Vec::new(),
            array: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_proto(proto: Value) -> Self {
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

    pub fn insert_array(&mut self, index: usize, value: Variable) {
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
            };
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
        self.properties.insert(
            property_key,
            len,
        );
    }

    #[must_use]
    pub fn resolve_array(&self, index: usize) -> Option<ObjectProperty> {
        let (i, found) = self.array_position(index);

        if found {
            return self.array.get(i).and_then(|v| {
                self.values.get(v.1).cloned()
            });
        }

        None
    }

    #[must_use]
    pub fn get_array(&self, index: usize) -> Option<&ObjectProperty> {
        let (i, found) = self.array_position(index);

        if found {
            return self.array.get(i).and_then(|v| {
                self.values.get(v.1)
            });
        }

        None
    }

    pub fn delete_array(&mut self, index: usize) -> Option<Value> {
        let (i, found) = self.array_position(index);

        if found
            && self
            .array
            .get(i)
            .is_some_and(|v| {
                let Some(v) = self.values.get(v.1) else {
                    return false;
                };

                v.attributes.is_configurable()
            })
        {

            let idx = self.array.remove(i);

            return self.values
                .get_mut(idx.1)
                .map(|v| {
                    mem::replace(&mut v.value, Value::Undefined)
                });
        }

        None
    }

    pub fn set_array(&mut self, elements: Vec<Value>) {
        self.array.clear();

        let len = self.values.len();
        let elements_len = elements.len();

        self.values.extend(elements.into_iter().map(ObjectProperty::new));

        for i in 0..elements_len {
            self.array.push((i, len + i));
        }
    }

    pub fn get_array_mut(&mut self, index: usize) -> Option<&mut Value> {
        let (i, found) = self.array_position(index);

        if found {
            return self.array.get(i)
                .and_then(|v| {
                    self.values.get_mut(v.1)
                        .map(|p| &mut p.value)
                }); //TODO: Check for perms
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

    pub fn from_values(values: Vec<(Value, Value)>, realm: &Realm) -> Result<Self, Error> {
        let mut object = Self::new(realm);

        for (key, value) in values {
            object.define_property(key, value)?;
        }

        Ok(object)
    }

    pub fn force_update_property_cb(&mut self, name: Value, cb: impl FnOnce(Option<&mut ObjectProperty>) -> Option<Value>) -> Res {
        let key = name.into();


        match self.properties.entry(key) {
            Entry::Occupied(entry) => {
                let idx = *entry.get();

                let Some(val) = cb(self.values.get_mut(idx)) else {
                    return Ok(());
                };

                if let Some(v) = self.values.get_mut(idx) {
                    if v.attributes.is_writable() {
                        v.value = val;
                    }
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

}

impl MutObj<Realm> for MutObject {
    fn define_property(&mut self, name: Value, value: Value) -> Result<(), Error> {
        let key = InternalPropertyKey::from(name);


        if let InternalPropertyKey::Index(n) = key {
            self.insert_array(n, value.into());
            return Ok(());
        }

        if let InternalPropertyKey::String(s) = &key {
            if s == "__proto__" {
                self.prototype = value.into();
                return Ok(());
            }
        }

        match self.properties.entry(key.into()) {
            Entry::Occupied(entry) => {
                let Some(e) = self.values.get_mut(*entry.get()) else {
                    return Err(Error::new("Failed to get value for property"));
                };

                if e.attributes.is_writable() {
                    e.value = value;
                    return Ok(());
                }
            }
            Entry::Vacant(entry) => {

                let idx = self.values.len();
                self.values.push(ObjectProperty::new(value));
                entry.insert(idx);
            }
        }
        Ok(())
    }

    fn define_variable(&mut self, name: Value, value: Variable) -> Result<(), Error> {
        let key = InternalPropertyKey::from(name);

        if let InternalPropertyKey::Index(n) = key {
            self.insert_array(n, value);
            return Ok(());
        }

        if let InternalPropertyKey::String(s) = &key {
            if s == "__proto__" {
                self.prototype = value.into();
                return Ok(());
            }
        }

        match self.properties.entry(key.into()) {
            Entry::Occupied(entry) => {
                let Some(e) = self.values.get_mut(*entry.get()) else {
                    return Err(Error::new("Failed to get value for property"));
                };

                if e.attributes.is_writable() {
                    *e = value.into();

                    return Ok(());
                }
            }
            Entry::Vacant(entry) => {
                let idx = self.values.len();
                self.values.push(value.into());
                entry.insert(idx);
            }
        }

        Ok(())
    }

    fn resolve_property(&self, name: &Value) -> Result<Option<ObjectProperty>, Error> {
        let key = InternalPropertyKey::from(name.clone());


        if matches!(&key, InternalPropertyKey::String(str) if str == "__proto__") {
            return Ok(Some(self.prototype.clone()));
        }

        if let InternalPropertyKey::Index(n) = key {
            if let Some(value) = self.resolve_array(n) {
                return Ok(Some(value));
            }
            //TODO: we should insert a new reference in the array to the value if we find it in the property map
        }


        Ok(self
            .properties
            .get(&key.into())
            .and_then(|idx| self.values.get(*idx))
            .cloned()
            .or_else(|| match &self.prototype.value {
                Value::Object(o) => o.resolve_property_no_get_set(name).ok().flatten(), //TODO: this is wrong, we need a realm here!
                _ => None,
            }))
    }

    fn get_property(&self, name: &Value) -> Result<Option<ObjectProperty>, Error> {
        let key = InternalPropertyKey::from(name.clone());


        if matches!(&key, InternalPropertyKey::String(str) if str == "__proto__") {
            return Ok(Some(self.prototype.clone()));
        }

        if let Value::Number(n) = name {
            return Ok(self.get_array(*n as usize).cloned());
        }

        if let Some(prop) = self.properties.get(&key.into()) {
            return Ok(self.values.get(*prop).cloned());
        }

        Ok(None)
    }

    fn define_getter(&mut self, name: Value, value: Value) -> Res {
        let key = PropertyKey::from(name.clone());

        let val = self.properties.get_mut(&key)
            .and_then(|idx| self.values.get_mut(*idx));

        if let Some(prop) = val {
            prop.get = value;
            return Ok(());
        }

        let len = self.values.len();
        self.values.push(ObjectProperty::getter(value));
        self.properties.insert(key, len);

        Ok(())
    }

    fn define_setter(&mut self, name: Value, value: Value) -> Res {
        let key = PropertyKey::from(name.clone());

        let val = self.properties.get_mut(&key)
            .and_then(|idx| self.values.get_mut(*idx));



        if let Some(prop) = val {
            prop.set = value;
            return Ok(());
        }

        let len = self.values.len();
        self.values.push(ObjectProperty::setter(value));
        self.properties.insert(key, len);


        Ok(())
    }

    fn delete_property(&mut self, name: &Value) -> Result<Option<Value>, Error> {
        let key = InternalPropertyKey::from(name.clone());


        if matches!(&key, InternalPropertyKey::String(str) if str == "__proto__") {
            return Ok(None);
        }

        if let InternalPropertyKey::Index(n) = key {
            return Ok(self.delete_array(n));
        }

        if let Entry::Occupied(occ) = self.properties.entry(key.into()) {
            let prop = self.values.get_mut(*occ.get())
                .ok_or_else(|| Error::new("Failed to get value for property"))?;


            return if prop.attributes.is_configurable() {
                Ok(Some(mem::replace(&mut prop.value, Value::Undefined)))
            } else {
                // Err(Error::ty("Property is not configurable")) // this is only in strict mode
                Ok(None)
            };
        }

        Ok(None)
    }

    fn contains_key(&self, name: &Value) -> Result<bool, Error> {
        let name = InternalPropertyKey::from(name.clone());

        if matches!(&name, InternalPropertyKey::String(str) if str == "__proto__") {
            return Ok(true);
        }

        if let InternalPropertyKey::Index(n) = name {
            return Ok(self.contains_array_key(n));
        }

        Ok(self.properties.contains_key(&name.into()))
    }

    fn name(&self) -> String {
        "Object".to_string()
    }

    fn to_string(&self, _realm: &mut Realm) -> Result<YSString, Error> {
        Ok("[object Object]".into())
    }

    fn to_string_internal(&self) -> Result<YSString, Error> {
        Ok("[object Object]".into())
    }

    fn properties(&self) -> Result<Vec<(Value, Value)>, Error> {
        Ok(self.
                properties
                    .iter()
                    .filter_map(|(k, v)| {
                        let v = self.values.get(*v)?;

                        Some((k.clone().into(), v.value.copy()))
                    })
            .collect()
        )
    }

    fn keys(&self) -> Result<Vec<Value>, Error> {
        Ok(self.properties.keys()
            .cloned()
            .map(Into::into)
            .collect()
        )
    }

    fn values(&self) -> Result<Vec<Value>, Error> {
        Ok(self.values.iter()
            .map(|v| v.value.copy())
            .collect())
        //TODO: getter (and setter) values
    }

    fn get_array_or_done(&self, index: usize) -> Result<(bool, Option<Value>), Error> {
        if let Some(value) = self.resolve_array(index) {
            let done = if let Some((i, _)) = self.array.last() {
                index > *i
            } else {
                true
            };
            return Ok((done, Some(value.value)));
        }

        Ok((true, None))
    }

    fn clear_values(&mut self) -> Res {
        self.properties.clear();
        self.array.clear();

        Ok(())
    }

    fn prototype(&self) -> Result<ObjectProperty, Error> {
        Ok(self.prototype.clone())
    }

    fn set_prototype(&mut self, proto: ObjectProperty) -> Res {
        self.prototype = proto;
        Ok(())
    }

    fn constructor(&self) -> Result<ObjectProperty, Error> {
        if let Some(constructor) = self.properties.get(&PropertyKey::String("constructor".into())) {
            if let Some(value) = self.values.get(*constructor) {
                return Ok(value.clone());
            }
        }

        if let Value::Object(proto) = self.prototype()?.value {
            let proto = proto.guard();

            return proto.constructor();
        }

        Ok(Value::Undefined.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::Value;

    use super::*;

    #[test]
    fn object_creation_with_proto() {
        let proto = Value::Number(42.0);
        let object = Object::with_proto(proto.clone());

        assert_eq!(
            object.get_property(&"__proto__".into()).unwrap().value,
            proto
        );
    }

    #[test]
    fn object_creation_raw_with_proto() {
        let proto = Value::Number(42.0);
        let object = Object::raw_with_proto(proto.copy());

        assert_eq!(object.prototype().unwrap().value, proto);
    }

    #[test]
    fn array_position_empty_array() {
        let realm = Realm::new().unwrap();
        let object = Object::raw(&realm);

        let (index, found) = object.inner().unwrap().array_position(0);

        assert_eq!(index, 0);
        assert!(!found);
    }

    #[test]
    fn array_position_non_empty_array() {
        let realm = Realm::new().unwrap();
        let object = Object::raw(&realm);
        object
            .inner_mut()
            .unwrap()
            .insert_array(0, Value::Number(42.0).into());

        let (index, found) = object.inner().unwrap().array_position(0);

        assert_eq!(index, 0);
        assert!(found);
    }

    #[test]
    fn insert_array() {
        let realm = Realm::new().unwrap();
        let object = Object::raw(&realm);
        object
            .inner_mut()
            .unwrap()
            .insert_array(0, Value::Number(42.0).into());

        assert_eq!(
            object.inner().unwrap().array[0].1.value,
            Value::Number(42.0)
        );
    }

    #[test]
    fn resolve_array() {
        let realm = Realm::new().unwrap();
        let object = Object::raw(&realm);
        object
            .inner_mut()
            .unwrap()
            .insert_array(0, Value::Number(42.0).into());

        let value = object.inner().unwrap().resolve_array(0);

        assert_eq!(value, Some(Value::Number(42.0).into()));
    }

    #[test]
    fn get_array() {
        let realm = Realm::new().unwrap();
        let object = Object::raw(&realm);
        object
            .inner_mut()
            .unwrap()
            .insert_array(0, Value::Number(42.0).into());

        let inner = object.inner().unwrap();

        let value = inner.get_array(0).unwrap();

        assert_eq!(&value.value, &Value::Number(42.0));
    }

    #[test]
    fn get_array_mut() {
        let realm = Realm::new().unwrap();
        let object = Object::raw(&realm);
        object
            .inner_mut()
            .unwrap()
            .insert_array(0, Value::Number(42.0).into());

        let mut inner = object.inner_mut().unwrap();

        let value = inner.get_array_mut(0).unwrap();

        assert_eq!(value, &mut Value::Number(42.0));
    }

    #[test]
    fn contains_array_key() {
        let realm = Realm::new().unwrap();
        let object = Object::raw(&realm);
        object
            .inner_mut()
            .unwrap()
            .insert_array(0, Value::Number(42.0).into());

        let contains = object.inner().unwrap().contains_array_key(0);

        assert!(contains);
    }

    #[test]
    fn define_property() {
        let realm = Realm::new().unwrap();
        let object = Object::raw(&realm);
        object
            .define_property(Value::string("key"), Value::Number(42.0))
            .unwrap();

        assert_eq!(
            object
                .inner()
                .unwrap()
                .properties
                .get(&Value::string("key"))
                .unwrap()
                .value,
            Value::Number(42.0)
        );
    }

    #[test]
    fn resolve_property() {
        let realm = Realm::new().unwrap();
        let object = Object::raw(&realm);
        object
            .define_property(Value::string("key"), Value::Number(42.0))
            .unwrap();

        let value = object.resolve_property(&Value::string("key")).unwrap();

        assert_eq!(value, Some(Value::Number(42.0).into()));
    }

    #[test]
    fn get_property() {
        let realm = Realm::new().unwrap();
        let object = Object::raw(&realm);
        object
            .define_property(Value::string("key"), Value::Number(42.0))
            .unwrap();

        let value = object.get_property(&Value::string("key")).unwrap();

        assert_eq!(value.unwrap().value, Value::Number(42.0));
    }

    #[test]
    fn contains_key() {
        let realm = Realm::new().unwrap();
        let object = Object::raw(&realm);
        object
            .define_property(Value::string("key"), Value::Number(42.0))
            .unwrap();

        let contains = object.contains_key(&Value::string("key")).unwrap();

        assert!(contains);
    }
}
