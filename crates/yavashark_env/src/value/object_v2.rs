#![allow(warnings)]

use crate::value::property_key::InternalPropertyKey;
use std::any::TypeId;
use std::fmt::Debug;
use std::ptr::NonNull;
use yavashark_garbage::Collectable;

pub struct Realm;

type Value = ();
type PrimitiveValue = ();
type Res<T = ()> = Result<T, ()>;
type Variable = ();
type ObjectHandle = ();

type NullableObjectHandle = Option<ObjectHandle>;

type PreHashedPropertyKey = (InternalPropertyKey, u64);

pub trait ObjV2: Collectable + Debug + 'static {
    fn define_property(&self, name: InternalPropertyKey, value: Value, realm: &mut Realm) -> Res;
    fn define_property_attributes(
        &self,
        name: InternalPropertyKey,
        value: Variable,
        realm: &mut Realm,
    ) -> Res;

    fn resolve_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Variable>>;
    fn get_own_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Variable>>;

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
    ) -> Res<Option<Variable>>;

    fn contains_own_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool>;

    fn contains_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool>;

    fn define_property_pre_hash(
        &self,
        name: PreHashedPropertyKey,
        value: Value,
        realm: &mut Realm,
    ) -> Res {
        self.define_property(name.0, value, realm)
    }
    fn define_property_attributes_pre_hash(
        &self,
        name: PreHashedPropertyKey,
        value: Variable,
        realm: &mut Realm,
    ) -> Res {
        self.define_property_attributes(name.0, value, realm)
    }

    fn resolve_property_pre_hash(
        &self,
        name: PreHashedPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Variable>> {
        self.resolve_property(name.0, realm)
    }
    fn get_own_property_pre_hash(
        &self,
        name: PreHashedPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Variable>> {
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
    ) -> Res<Option<Variable>> {
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

    fn properties(&self, realm: &mut Realm) -> Res<Vec<(Value, Value)>>;
    fn keys(&self, realm: &mut Realm) -> Res<Vec<Value>>;
    fn values(&self, realm: &mut Realm) -> Res<Vec<Value>>;

    fn enumerable_properties(&self, realm: &mut Realm) -> Res<Vec<(Value, Value)>>;
    fn enumerable_keys(&self, realm: &mut Realm) -> Res<Vec<Value>>;
    fn enumerable_values(&self, realm: &mut Realm) -> Res<Vec<Value>>;

    fn clear_properties(&self, realm: &mut Realm) -> Res;

    fn get_array_or_done(&self, realm: &mut Realm) -> Res<(bool, Value)>;
    fn call(&self, this: Value, args: Vec<Value>, realm: &mut Realm) -> Res<Value>;
    fn is_callable(&self) -> bool;

    fn primitive(&self, realm: &mut Realm) -> Res<Option<PrimitiveValue>>;

    fn prototype(&self, realm: &mut Realm) -> Res<NullableObjectHandle>;
    fn set_prototype(&self, prototype: NullableObjectHandle, realm: &mut Realm) -> Res;

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
}
