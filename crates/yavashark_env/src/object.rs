use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::fmt::Debug;

pub use prototype::*;
use yavashark_garbage::GcRef;
use yavashark_value::{BoxedObj, MutObj, Obj};

use crate::realm::Realm;
use crate::{Error, ObjectHandle, ObjectProperty, Variable};
use crate::{Res, Value};

pub mod array;
mod constructor;
mod prototype;

#[derive(Debug)]
pub struct Object {
    inner: RefCell<MutObject>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MutObject {
    pub properties: HashMap<Value, ObjectProperty>,
    pub array: Vec<(usize, ObjectProperty)>,
    pub prototype: ObjectProperty,
}

impl Object {
    #[must_use]
    #[allow(clippy::new_ret_no_self)]
    pub fn new(realm: &Realm) -> ObjectHandle {
        ObjectHandle::new(Self::raw(realm))
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

    fn get_getter(&self, name: &Value) -> Result<Option<Value>, Error> {
        self.inner()?.get_getter(name)
    }

    fn get_setter(&self, name: &Value) -> Result<Option<Value>, Error> {
        self.inner()?.get_setter(name)
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

    fn to_string(&self, realm: &mut Realm) -> Result<String, Error> {
        self.inner()?.to_string(realm)
    }

    fn to_string_internal(&self) -> Result<String, Error> {
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
            properties: HashMap::new(),
            prototype,
            array: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_proto(proto: Value) -> Self {
        Self {
            properties: HashMap::new(),
            prototype: proto.into(),
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
            if let Some(v) = self.array.get_mut(i) {
                v.1 = value.into();
                return;
            };
        }

        self.array.insert(i, (index, value.into()));
    }

    #[must_use]
    pub fn resolve_array(&self, index: usize) -> Option<ObjectProperty> {
        let (i, found) = self.array_position(index);

        if found {
            return self.array.get(i).map(|v| v.1.clone());
        }

        None
    }

    #[must_use]
    pub fn get_array(&self, index: usize) -> Option<&ObjectProperty> {
        let (i, found) = self.array_position(index);

        if found {
            return self.array.get(i).map(|v| &v.1);
        }

        None
    }

    pub fn delete_array(&mut self, index: usize) -> Option<Value> {
        let (i, found) = self.array_position(index);

        if found {
            return Some(self.array.remove(i).1.value);
        }

        None
    }

    pub fn set_array(&mut self, elements: Vec<Value>) {
        self.array.clear();
        for (i, v) in elements.into_iter().enumerate() {
            self.array.push((i, ObjectProperty::new(v)));
        }
    }

    pub fn get_array_mut(&mut self, index: usize) -> Option<&mut Value> {
        let (i, found) = self.array_position(index);

        if found {
            return self.array.get_mut(i).map(|v| &mut v.1.value); //TODO: Check for perms
        }

        None
    }

    #[must_use]
    pub fn contains_array_key(&self, index: usize) -> bool {
        let (_, found) = self.array_position(index);

        found
    }

    pub fn from_values(values: Vec<(Value, Value)>, realm: &Realm) -> Result<Self, Error> {
        let mut object = Self::new(realm);

        for (key, value) in values {
            object.define_property(key, value)?;
        }

        Ok(object)
    }
}

impl MutObj<Realm> for MutObject {
    fn define_property(&mut self, name: Value, value: Value) -> Result<(), Error> {
        if let Value::Number(n) = &name {
            self.insert_array(*n as usize, value.into());
            return Ok(());
        }

        self.properties.insert(name, value.into());

        Ok(())
    }

    fn define_variable(&mut self, name: Value, value: Variable) -> Result<(), Error> {
        if let Value::Number(n) = &name {
            self.insert_array(*n as usize, value);
            return Ok(());
        }
        self.properties.insert(name, value.into());

        Ok(())
    }

    fn resolve_property(&self, name: &Value) -> Result<Option<ObjectProperty>, Error> {
        if name == &Value::String("__proto__".to_string()) {
            return Ok(Some(self.prototype.clone()));
        }

        if let Value::Number(n) = name {
            return Ok(self.resolve_array(*n as usize));
        }

        Ok(self
            .properties
            .get(name)
            .cloned()
            .or_else(|| match &self.prototype.value {
                Value::Object(o) => o.resolve_property_no_get_set(name).ok().flatten(), //TODO: this is wrong, we need a realm here!
                _ => None,
            }))
    }

    fn get_property(&self, name: &Value) -> Result<Option<ObjectProperty>, Error> {
        if name == &Value::String("__proto__".to_string()) {
            return Ok(Some(self.prototype.copy()));
        }

        if let Value::Number(n) = name {
            return Ok(self.get_array(*n as usize).cloned());
        }

        if let Some(prop) = self.properties.get(name) {
            return Ok(Some(prop.copy()));
        }

        Ok(None)
    }

    fn define_getter(&mut self, name: Value, value: Value) -> Res {
        let val = self.properties.get_mut(&name);
        if let Some(prop) = val {
            prop.get = value;
            return Ok(());
        }

        self.properties.insert(name, ObjectProperty::getter(value));

        Ok(())
    }

    fn define_setter(&mut self, name: Value, value: Value) -> Res {
        let val = self.properties.get_mut(&name);
        if let Some(prop) = val {
            prop.set = value;
            return Ok(());
        }

        self.properties.insert(name, ObjectProperty::setter(value));

        Ok(())
    }

    fn get_getter(&self, _name: &Value) -> Result<Option<Value>, Error> {
        todo!("I guess, this can't be removed?")
    }

    fn get_setter(&self, _name: &Value) -> Result<Option<Value>, Error> {
        todo!("I guess, this can't be removed?")
    }

    fn delete_property(&mut self, name: &Value) -> Result<Option<Value>, Error> {
        if name == &Value::String("__proto__".to_string()) {
            return Ok(Some(self.prototype.value.clone()));
        }

        if let Value::Number(n) = name {
            return Ok(self.delete_array(*n as usize));
        }

        Ok(self.properties.remove(name).map(|e| e.value))
    }

    fn contains_key(&self, name: &Value) -> Result<bool, Error> {
        if name == &Value::String("__proto__".to_string()) {
            return Ok(true);
        }

        if let Value::Number(n) = name {
            return Ok(self.contains_array_key(*n as usize));
        }

        Ok(self.properties.contains_key(name))
    }

    fn name(&self) -> String {
        "Object".to_string()
    }

    fn to_string(&self, _realm: &mut Realm) -> Result<String, Error> {
        Ok("[object Object]".to_string())
    }

    fn to_string_internal(&self) -> Result<String, Error> {
        Ok("[object Object]".to_string())
    }

    fn properties(&self) -> Result<Vec<(Value, Value)>, Error> {
        Ok(self
            .array
            .iter()
            .map(|(i, v)| (Value::Number(*i as f64), v.value.copy()))
            .chain(
                self.properties
                    .iter()
                    .map(|(k, v)| (k.copy(), v.value.copy())),
            )
            .collect())
    }

    fn keys(&self) -> Result<Vec<Value>, Error> {
        Ok(self
            .array
            .iter()
            .map(|(i, _)| Value::Number(*i as f64))
            .chain(self.properties.keys().map(Value::copy))
            .collect())
    }

    fn values(&self) -> Result<Vec<Value>, Error> {
        Ok(self
            .array
            .iter()
            .map(|(_, v)| v.value.copy())
            .chain(self.properties.values().map(|v| v.value.copy()))
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

    fn constructor(&self) -> Result<ObjectProperty, Error> {
        if let Some(constructor) = self
            .properties
            .get(&Value::String("constructor".to_string()))
        {
            return Ok(constructor.clone());
        }

        if let Value::Object(proto) = self.prototype()?.value {
            let proto = proto.get();

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
            .define_property(Value::String("key".to_string()), Value::Number(42.0))
            .unwrap();

        assert_eq!(
            object
                .inner()
                .unwrap()
                .properties
                .get(&Value::String("key".to_string()))
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
            .define_property(Value::String("key".to_string()), Value::Number(42.0))
            .unwrap();

        let value = object
            .resolve_property(&Value::String("key".to_string()))
            .unwrap();

        assert_eq!(value, Some(Value::Number(42.0).into()));
    }

    #[test]
    fn get_property() {
        let realm = Realm::new().unwrap();
        let object = Object::raw(&realm);
        object
            .define_property(Value::String("key".to_string()), Value::Number(42.0))
            .unwrap();

        let value = object
            .get_property(&Value::String("key".to_string()))
            .unwrap();

        assert_eq!(value.unwrap().value, Value::Number(42.0));
    }

    #[test]
    fn contains_key() {
        let realm = Realm::new().unwrap();
        let object = Object::raw(&realm);
        object
            .define_property(Value::String("key".to_string()), Value::Number(42.0))
            .unwrap();

        let contains = object
            .contains_key(&Value::String("key".to_string()))
            .unwrap();

        assert!(contains);
    }
}
