use yavashark_garbage::GcRef;
use crate::{ObjectHandle, Realm, Res, Value};
use crate::value::BoxedObj;
use crate::value::property_key::{InternalPropertyKey, PropertyKey};


pub enum UpdatePropertyResult {
    Handled,
    NotHandled(Value),
    Setter(ObjectHandle, Value),
    ReadOnly,
}

pub enum Property {
    Value(Value),
    Getter(ObjectHandle),
}

pub trait PropertiesHook {
    fn set_property(&self, key: &InternalPropertyKey, value: Value, realm: &mut Realm) -> Res<UpdatePropertyResult>;
    fn get_property(&self, key: &InternalPropertyKey) -> Res<Option<Property>>;

    fn contains_property(&self, key: &InternalPropertyKey) -> Res<bool> {
        Ok(self.get_property(key)?.is_some())
    }

    fn properties(&self) -> Res<impl Iterator<Item = (PropertyKey, Property)>>;
    fn keys(&self) -> Res<impl Iterator<Item = PropertyKey>>;
    fn values(&self) -> Res<impl Iterator<Item = Property>>;


    fn enumerable_properties(&self) -> Res<impl Iterator<Item = (PropertyKey, Property)>> {
        self.properties()
    }
    fn enumerable_keys(&self) -> Res<impl Iterator<Item = PropertyKey>> {
        self.keys()
    }
    fn enumerable_values(&self) -> Res<impl Iterator<Item = Property>> {
        self.values()
    }

    fn gc_refs(&self) -> impl Iterator<Item = GcRef<BoxedObj>>;
}