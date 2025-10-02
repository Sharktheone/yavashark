use yavashark_garbage::GcRef;
use crate::{Error, Res, Value};
use crate::value::BoxedObj;
use crate::value::property_key::{InternalPropertyKey, PropertyKey};


pub enum UpdatePropertyResult {
    Handled,
    NotHandled(Value),
    Invalid,
}

pub trait PropertiesHook {
    fn set_property(&self, key: InternalPropertyKey, value: Value) -> Res<UpdatePropertyResult>;
    fn get_property(&self, key: InternalPropertyKey) -> Res<Option<Value>>;

    fn contains_property(&self, key: InternalPropertyKey) -> Res<bool> {
        Ok(self.get_property(key)?.is_some())
    }

    fn properties(&self) -> Res<impl Iterator<Item = (PropertyKey, Value)>>;
    fn keys(&self) -> Res<impl Iterator<Item = PropertyKey>>;
    fn values(&self) -> Res<impl Iterator<Item = Value>>;


    fn enumerable_properties(&self) -> Res<impl Iterator<Item = (PropertyKey, Value)>> {
        self.properties()
    }
    fn enumerable_keys(&self) -> Res<impl Iterator<Item = PropertyKey>> {
        self.keys()
    }
    fn enumerable_values(&self) -> Res<impl Iterator<Item = Value>> {
        self.values()
    }

    fn gc_refs(&self) -> impl Iterator<Item = GcRef<BoxedObj>>;
}