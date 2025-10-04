use crate::error::Error;
use crate::value::{BoxedObj, DefinePropertyResult, MutObj, Obj, Object, ObjectProperty, Property, Value, Variable};
use crate::{InternalPropertyKey, ObjectHandle, ObjectOrNull, PreHashedPropertyKey, PrimitiveValue, PropertyKey, Realm, Res};
use std::any::TypeId;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use yavashark_garbage::GcRef;
use yavashark_string::YSString;

pub trait ObjectImpl: Debug + 'static {
    type Inner;

    /// the returned object should NOT be a reference to self, but a reference to the object that is wrapped by self
    fn get_wrapped_object(&self) -> impl DerefMut<Target = impl MutObj>;

    fn get_inner(&self) -> impl Deref<Target = Self::Inner>;

    fn get_inner_mut(&self) -> impl DerefMut<Target = Self::Inner>;



    fn define_property(
        &self,
        name: InternalPropertyKey,
        value: Value,
        realm: &mut Realm,
    ) -> Res<DefinePropertyResult> {
        self.get_wrapped_object()
            .define_property(name, value, realm)
    }
    fn define_property_attributes(
        &self,
        name: InternalPropertyKey,
        value: Variable,
        realm: &mut Realm,
    ) -> Res<DefinePropertyResult> {
        self.get_wrapped_object()
            .define_property_attributes(name, value, realm)
    }

    fn resolve_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        self.get_wrapped_object().resolve_property(name, realm)
    }
    fn get_own_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        self.get_wrapped_object().get_own_property(name, realm)
    }

    fn define_getter(
        &self,
        name: InternalPropertyKey,
        callback: ObjectHandle,
        realm: &mut Realm,
    ) -> Res {
        self.get_wrapped_object()
            .define_getter(name, callback, realm)
    }
    fn define_setter(
        &self,
        name: InternalPropertyKey,
        callback: ObjectHandle,
        realm: &mut Realm,
    ) -> Res {
        self.get_wrapped_object()
            .define_setter(name, callback, realm)
    }

    fn delete_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        self.get_wrapped_object().delete_property(name, realm)
    }

    fn contains_own_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool> {
        self.get_wrapped_object().contains_own_key(name, realm)
    }

    fn contains_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool> {
        self.get_wrapped_object().contains_key(name, realm)
    }

    fn define_property_pre_hash(
        &self,
        name: PreHashedPropertyKey,
        value: Value,
        realm: &mut Realm,
    ) -> Res<DefinePropertyResult> {
        self.get_wrapped_object()
            .define_property_pre_hash(name, value, realm)
    }
    fn define_property_attributes_pre_hash(
        &self,
        name: PreHashedPropertyKey,
        value: Variable,
        realm: &mut Realm,
    ) -> Res<DefinePropertyResult> {
        self.get_wrapped_object()
            .define_property_attributes_pre_hash(name, value, realm)
    }

    fn resolve_property_pre_hash(
        &self,
        name: PreHashedPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        self.get_wrapped_object()
            .resolve_property_pre_hash(name, realm)
    }
    fn get_own_property_pre_hash(
        &self,
        name: PreHashedPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        self.get_wrapped_object()
            .get_own_property_pre_hash(name, realm)
    }

    fn define_getter_pre_hash(
        &self,
        name: PreHashedPropertyKey,
        callback: ObjectHandle,
        realm: &mut Realm,
    ) -> Res {
        self.get_wrapped_object()
            .define_getter_pre_hash(name, callback, realm)
    }
    fn define_setter_pre_hash(
        &self,
        name: PreHashedPropertyKey,
        callback: ObjectHandle,
        realm: &mut Realm,
    ) -> Res {
        self.get_wrapped_object()
            .define_setter_pre_hash(name, callback, realm)
    }

    fn delete_property_pre_hash(
        &self,
        name: PreHashedPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        self.get_wrapped_object()
            .delete_property_pre_hash(name, realm)
    }

    fn contains_own_key_pre_hash(
        &self,
        name: PreHashedPropertyKey,
        realm: &mut Realm,
    ) -> Res<bool> {
        self.get_wrapped_object()
            .contains_own_key_pre_hash(name, realm)
    }

    fn contains_key_pre_hash(&self, name: PreHashedPropertyKey, realm: &mut Realm) -> Res<bool> {
        self.get_wrapped_object()
            .contains_key_pre_hash(name, realm)
    }

    fn properties(&self, realm: &mut Realm) -> Res<Vec<(PropertyKey, Value)>> {
        self.get_wrapped_object().properties(realm)
    }
    fn keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>> {
        self.get_wrapped_object().keys(realm)
    }
    fn values(&self, realm: &mut Realm) -> Res<Vec<Value>> {
        self.get_wrapped_object().values(realm)
    }

    fn enumerable_properties(&self, realm: &mut Realm) -> Res<Vec<(PropertyKey, Value)>> {
        self.get_wrapped_object().enumerable_properties(realm)
    }
    fn enumerable_keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>> {
        self.get_wrapped_object().enumerable_keys(realm)
    }
    fn enumerable_values(&self, realm: &mut Realm) -> Res<Vec<Value>> {
        self.get_wrapped_object().enumerable_values(realm)
    }

    fn clear_properties(&self, realm: &mut Realm) -> Res {
        self.get_wrapped_object().clear_properties(realm)
    }

    fn get_array_or_done(&self, idx: usize, realm: &mut Realm) -> Res<(bool, Option<Value>)> {
        self.get_wrapped_object().get_array_or_done(idx, realm)
    }
    fn call(&self, args: Vec<Value>, this: Value, realm: &mut Realm) -> Res<Value> {
        self.get_wrapped_object().call(args, this, realm)
    }
    fn is_callable(&self) -> bool {
        self.get_wrapped_object().is_callable()
    }

    fn primitive(&self, realm: &mut Realm) -> Res<Option<PrimitiveValue>> {
        self.get_wrapped_object().primitive(realm)
    }

    fn prototype(&self, realm: &mut Realm) -> Res<ObjectOrNull> {
        self.get_wrapped_object().prototype(realm)
    }
    fn set_prototype(&self, prototype: ObjectOrNull, realm: &mut Realm) -> Res {
        self.get_wrapped_object().set_prototype(prototype, realm)
    }

    fn construct(&self, args: Vec<Value>, realm: &mut Realm) -> Res<ObjectHandle> {
        self.get_wrapped_object().construct(args, realm)
    }
    fn is_constructable(&self) -> bool {
        self.get_wrapped_object().is_constructable()
    }

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
            self.get_wrapped_object().inner_downcast(ty)
        }
    }

    unsafe fn inner_downcast_fat_ptr(&self, ty: TypeId) -> Option<NonNull<[()]>> {
        self.get_wrapped_object().inner_downcast_fat_ptr(ty)
    }

    fn is_extensible(&self) -> bool {
        self.get_wrapped_object().is_extensible()
    }

    fn prevent_extensions(&self) -> Res {
        self.get_wrapped_object().prevent_extensions()
    }

    fn is_frozen(&self) -> bool {
        self.get_wrapped_object().is_frozen()
    }

    fn freeze(&self) -> Res {
        self.get_wrapped_object().freeze()
    }

    fn is_sealed(&self) -> bool {
        self.get_wrapped_object().is_sealed()
    }

    fn seal(&self) -> Res {
        self.get_wrapped_object().seal()
    }


    fn gc_refs(&self) -> Vec<GcRef<BoxedObj>> {
        self.get_wrapped_object().gc_refs()
    }

}

impl<T: ObjectImpl> Obj for T {
    fn define_property(&self, name: InternalPropertyKey, value: Value, realm: &mut Realm) -> Res<DefinePropertyResult> {
        ObjectImpl::define_property(self, name, value, realm)
    }

    fn define_property_attributes(&self, name: InternalPropertyKey, value: Variable, realm: &mut Realm) -> Res<DefinePropertyResult> {
        ObjectImpl::define_property_attributes(self, name, value, realm)
    }

    fn resolve_property(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<Option<Property>> {
        ObjectImpl::resolve_property(self, name, realm)
    }

    fn get_own_property(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<Option<Property>> {
        ObjectImpl::get_own_property(self, name, realm)
    }

    fn define_getter(&self, name: InternalPropertyKey, callback: ObjectHandle, realm: &mut Realm) -> Res {
        ObjectImpl::define_getter(self, name, callback, realm)
    }

    fn define_setter(&self, name: InternalPropertyKey, callback: ObjectHandle, realm: &mut Realm) -> Res {
        ObjectImpl::define_setter(self, name, callback, realm)
    }

    fn delete_property(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<Option<Property>> {
        ObjectImpl::delete_property(self, name, realm)
    }

    fn contains_own_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool> {
        ObjectImpl::contains_own_key(self, name, realm)
    }

    fn contains_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool> {
        ObjectImpl::contains_key(self, name, realm)
    }

    fn define_property_pre_hash(&self, name: PreHashedPropertyKey, value: Value, realm: &mut Realm) -> Res<DefinePropertyResult> {
        ObjectImpl::define_property_pre_hash(self, name, value, realm)
    }

    fn define_property_attributes_pre_hash(&self, name: PreHashedPropertyKey, value: Variable, realm: &mut Realm) -> Res<DefinePropertyResult> {
        ObjectImpl::define_property_attributes_pre_hash(self, name, value, realm)
    }

    fn resolve_property_pre_hash(&self, name: PreHashedPropertyKey, realm: &mut Realm) -> Res<Option<Property>> {
        ObjectImpl::resolve_property_pre_hash(self, name, realm)
    }

    fn get_own_property_pre_hash(&self, name: PreHashedPropertyKey, realm: &mut Realm) -> Res<Option<Property>> {
        ObjectImpl::get_own_property_pre_hash(self, name, realm)
    }

    fn define_getter_pre_hash(&self, name: PreHashedPropertyKey, callback: ObjectHandle, realm: &mut Realm) -> Res {
        ObjectImpl::define_getter_pre_hash(self, name, callback, realm)
    }

    fn define_setter_pre_hash(&self, name: PreHashedPropertyKey, callback: ObjectHandle, realm: &mut Realm) -> Res {
        ObjectImpl::define_setter_pre_hash(self, name, callback, realm)
    }

    fn delete_property_pre_hash(&self, name: PreHashedPropertyKey, realm: &mut Realm) -> Res<Option<Property>> {
        ObjectImpl::delete_property_pre_hash(self, name, realm)
    }

    fn contains_own_key_pre_hash(&self, name: PreHashedPropertyKey, realm: &mut Realm) -> Res<bool> {
        ObjectImpl::contains_own_key_pre_hash(self, name, realm)
    }

    fn contains_key_pre_hash(&self, name: PreHashedPropertyKey, realm: &mut Realm) -> Res<bool> {
        ObjectImpl::contains_key_pre_hash(self, name, realm)
    }

    fn properties(&self, realm: &mut Realm) -> Res<Vec<(PropertyKey, Value)>> {
        ObjectImpl::properties(self, realm)
    }

    fn keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>> {
        ObjectImpl::keys(self, realm)
    }

    fn values(&self, realm: &mut Realm) -> Res<Vec<Value>> {
        ObjectImpl::values(self, realm)
    }

    fn enumerable_properties(&self, realm: &mut Realm) -> Res<Vec<(PropertyKey, Value)>> {
        ObjectImpl::enumerable_properties(self, realm)
    }

    fn enumerable_keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>> {
        ObjectImpl::enumerable_keys(self, realm)
    }

    fn enumerable_values(&self, realm: &mut Realm) -> Res<Vec<Value>> {
        ObjectImpl::enumerable_values(self, realm)
    }

    fn clear_properties(&self, realm: &mut Realm) -> Res {
        ObjectImpl::clear_properties(self, realm)
    }

    fn get_array_or_done(&self, idx: usize, realm: &mut Realm) -> Res<(bool, Option<Value>)> {
        ObjectImpl::get_array_or_done(self, idx, realm)
    }

    fn call(&self, args: Vec<Value>, this: Value, realm: &mut Realm) -> Res<Value> {
        ObjectImpl::call(self, args, this, realm)
    }

    fn is_callable(&self) -> bool {
        ObjectImpl::is_callable(self)
    }

    fn primitive(&self, realm: &mut Realm) -> Res<Option<PrimitiveValue>> {
        ObjectImpl::primitive(self, realm)
    }

    fn prototype(&self, realm: &mut Realm) -> Res<ObjectOrNull> {
        ObjectImpl::prototype(self, realm)
    }

    fn set_prototype(&self, prototype: ObjectOrNull, realm: &mut Realm) -> Res {
        ObjectImpl::set_prototype(self, prototype, realm)
    }

    fn construct(&self, args: Vec<Value>, realm: &mut Realm) -> Res<ObjectHandle> {
        ObjectImpl::construct(self, args, realm)
    }

    fn is_constructable(&self) -> bool {
        ObjectImpl::is_constructable(self)
    }

    fn class_name(&self) -> &'static str {
        ObjectImpl::class_name(self)
    }

    unsafe fn inner_downcast(&self, ty: TypeId) -> Option<NonNull<()>> {
        ObjectImpl::inner_downcast(self, ty)
    }

    unsafe fn inner_downcast_fat_ptr(&self, ty: TypeId) -> Option<NonNull<[()]>> {
        ObjectImpl::inner_downcast_fat_ptr(self, ty)
    }

    fn is_extensible(&self) -> bool {
        ObjectImpl::is_extensible(self)
    }

    fn prevent_extensions(&self) -> Res {
        ObjectImpl::prevent_extensions(self)
    }

    fn is_frozen(&self) -> bool {
        ObjectImpl::is_frozen(self)
    }

    fn freeze(&self) -> Res {
        ObjectImpl::freeze(self)
    }

    fn is_sealed(&self) -> bool {
        ObjectImpl::is_sealed(self)
    }

    fn seal(&self) -> Res {
        ObjectImpl::seal(self)
    }

    fn gc_refs(&self) -> Vec<GcRef<BoxedObj>> {
        ObjectImpl::gc_refs(self)
    }
}
