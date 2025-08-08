use yavashark_value::property_key::InternalPropertyKey;
use crate::{ObjectProperty, Res};

pub struct ObjectProperties<H: Hooks> {
    pub(crate) hooks: H,
    props: super::ObjectProperties
}

pub trait Hooks {
    fn clear(&mut self);
    fn is_empty(&self) -> bool;

    fn insert(&mut self, key: &InternalPropertyKey, value: ObjectProperty);
    fn get(&self, key: &InternalPropertyKey) -> Option<&ObjectProperty>;
    fn remove(&mut self, key: &InternalPropertyKey) -> Res;
    fn contains_key(&self, key: &InternalPropertyKey) -> bool;

    // TODO: iterators
}