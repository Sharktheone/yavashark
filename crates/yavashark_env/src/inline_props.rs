use crate::value::property_key::{InternalPropertyKey, PropertyKey};
use crate::value::{Property, BoxedObj};
use crate::{ObjectHandle, Realm, Res, Value};
use yavashark_garbage::GcRef;

pub enum UpdatePropertyResult {
    Handled,
    NotHandled(Value),
    Setter(ObjectHandle, Value),
    ReadOnly,
}

pub trait PropertiesHook {
    fn set_property(
        &self,
        key: &InternalPropertyKey,
        value: Value,
        realm: &mut Realm,
    ) -> Res<UpdatePropertyResult>;
    fn get_property(&self, key: &InternalPropertyKey, realm: &mut Realm) -> Res<Option<Property>>;

    fn contains_property(&self, key: &InternalPropertyKey) -> Res<bool>;

    fn properties(&self, realm: &mut Realm) -> Res<impl Iterator<Item = (PropertyKey, Property)>>;
    fn keys(&self, realm: &mut Realm) -> Res<impl Iterator<Item = PropertyKey>>;
    fn values(&self, realm: &mut Realm) -> Res<impl Iterator<Item = Property>>;

    fn enumerable_properties(
        &self,
        realm: &mut Realm,
    ) -> Res<impl Iterator<Item = (PropertyKey, Property)>> {
        self.properties(realm)
    }
    fn enumerable_keys(&self, realm: &mut Realm) -> Res<impl Iterator<Item = PropertyKey>> {
        self.keys(realm)
    }
    fn enumerable_values(&self, realm: &mut Realm) -> Res<impl Iterator<Item = Property>> {
        self.values(realm)
    }
    fn delete_property(
        &self,
        key: &InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<bool>;

    fn gc_refs(&self) -> impl Iterator<Item = GcRef<BoxedObj>>;
}
