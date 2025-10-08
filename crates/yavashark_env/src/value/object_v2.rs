#![allow(warnings)]

use crate::value::property_key::{InternalPropertyKey, PropertyKey};
use crate::value::{BoxedObj, ObjectOrNull, PrimitiveValue, Value, Variable};
use crate::{ObjectHandle, PreHashedPropertyKey, Realm, Res};
use std::any::TypeId;
use std::fmt::Debug;
use std::ptr::NonNull;
use yavashark_garbage::GcRef;

pub enum DefinePropertyResult {
    Handled,
    ReadOnly,
    Setter(ObjectHandle, Value),
}

pub enum Property {
    Value(Variable),
    Getter(ObjectHandle),
}

pub trait ObjV2: Debug + 'static {
    fn define_property(
        &self,
        name: InternalPropertyKey,
        value: Value,
        realm: &mut Realm,
    ) -> Res<DefinePropertyResult>;
    fn define_property_attributes(
        &self,
        name: InternalPropertyKey,
        value: Variable,
        realm: &mut Realm,
    ) -> Res<DefinePropertyResult>;

    fn resolve_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>>;
    fn get_own_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>>;

    fn define_getter(
        &self,
        name: InternalPropertyKey,
        callback: ObjectHandle,
        realm: &mut Realm,
    ) -> Res;
    fn define_setter(
        &self,
        name: InternalPropertyKey,
        callback: ObjectHandle,
        realm: &mut Realm,
    ) -> Res;

    fn delete_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>>;

    fn contains_own_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool>;

    fn contains_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool>;

    fn define_property_pre_hash(
        &self,
        name: PreHashedPropertyKey,
        value: Value,
        realm: &mut Realm,
    ) -> Res<DefinePropertyResult> {
        self.define_property(name.0, value, realm)
    }
    fn define_property_attributes_pre_hash(
        &self,
        name: PreHashedPropertyKey,
        value: Variable,
        realm: &mut Realm,
    ) -> Res<DefinePropertyResult> {
        self.define_property_attributes(name.0, value, realm)
    }

    fn resolve_property_pre_hash(
        &self,
        name: PreHashedPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        self.resolve_property(name.0, realm)
    }
    fn get_own_property_pre_hash(
        &self,
        name: PreHashedPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        self.get_own_property(name.0, realm)
    }

    fn define_getter_pre_hash(
        &self,
        name: PreHashedPropertyKey,
        callback: ObjectHandle,
        realm: &mut Realm,
    ) -> Res {
        self.define_getter(name.0, callback, realm)
    }
    fn define_setter_pre_hash(
        &self,
        name: PreHashedPropertyKey,
        callback: ObjectHandle,
        realm: &mut Realm,
    ) -> Res {
        self.define_setter(name.0, callback, realm)
    }

    fn delete_property_pre_hash(
        &self,
        name: PreHashedPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        self.delete_property(name.0, realm)
    }

    fn contains_own_key_pre_hash(
        &self,
        name: PreHashedPropertyKey,
        realm: &mut Realm,
    ) -> Res<bool> {
        self.contains_own_key(name.0, realm)
    }

    fn contains_key_pre_hash(&self, name: PreHashedPropertyKey, realm: &mut Realm) -> Res<bool> {
        self.contains_key(name.0, realm)
    }

    fn properties(&self, realm: &mut Realm) -> Res<Vec<(PropertyKey, Value)>>;
    fn keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>>;
    fn values(&self, realm: &mut Realm) -> Res<Vec<Value>>;

    fn enumerable_properties(&self, realm: &mut Realm) -> Res<Vec<(PropertyKey, Value)>>;
    fn enumerable_keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>>;
    fn enumerable_values(&self, realm: &mut Realm) -> Res<Vec<Value>>;

    fn clear_properties(&self, realm: &mut Realm) -> Res;

    fn get_array_or_done(&self, realm: &mut Realm) -> Res<(bool, Value)>;
    fn call(&self, this: Value, args: Vec<Value>, realm: &mut Realm) -> Res<Value>;
    fn is_callable(&self) -> bool;

    fn primitive(&self, realm: &mut Realm) -> Res<Option<PrimitiveValue>>;

    fn prototype(&self, realm: &mut Realm) -> Res<ObjectOrNull>;
    fn set_prototype(&self, prototype: ObjectOrNull, realm: &mut Realm) -> Res;

    fn construct(&self, args: Vec<Value>, realm: &mut Realm) -> Res<ObjectHandle>; //TODO: i think this somehow needs to work differently
    fn is_constructable(&self) -> bool;

    fn class_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// # Safety
    /// - Caller and implementer must ensure that the pointer is a valid pointer to the type which the type id represents
    /// - Caller and implementer must ensure that the pointer is valid for the same lifetime of self
    unsafe fn inner_downcast(&self, ty: TypeId) -> Option<NonNull<()>> {
        if ty == TypeId::of::<Self>() {
            Some(NonNull::from(self).cast())
        } else {
            None
        }
    }

    unsafe fn inner_downcast_fat_ptr(&self, ty: TypeId) -> Option<NonNull<[()]>> {
        _ = ty;
        None
    }

    fn is_extensible(&self) -> bool {
        true
    }

    fn prevent_extensions(&self) -> Res;

    fn is_frozen(&self) -> bool {
        false
    }

    fn freeze(&self) -> Res;

    fn is_sealed(&self) -> bool {
        false
    }

    fn seal(&self) -> Res;

    fn gc_refs(&self) -> Vec<GcRef<BoxedObj>>;
}
