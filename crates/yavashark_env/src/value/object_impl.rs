use std::any::TypeId;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use yavashark_garbage::GcRef;
use yavashark_string::YSString;
use crate::error::Error;
use crate::Realm;
use crate::value::{AsAny, BoxedObj, MutObj, Obj, Object, ObjectProperty, Value, Variable};

pub trait ObjectImpl: Debug + AsAny + 'static {
    type Inner;

    /// the returned object should NOT be a reference to self, but a reference to the object that is wrapped by self
    fn get_wrapped_object(&self) -> impl DerefMut<Target = impl MutObj>;

    fn get_inner(&self) -> impl Deref<Target = Self::Inner>;

    fn get_inner_mut(&self) -> impl DerefMut<Target = Self::Inner>;

    fn define_property(&self, name: Value, value: Value) -> Result<(), Error> {
        self.get_wrapped_object().define_property(name, value)
    }

    fn define_variable(&self, name: Value, value: Variable) -> Result<(), Error> {
        self.get_wrapped_object().define_variable(name, value)
    }

    fn resolve_property(&self, name: &Value) -> Result<Option<ObjectProperty>, Error> {
        self.get_wrapped_object().resolve_property(name)
    }

    fn get_property(&self, name: &Value) -> Result<Option<ObjectProperty>, Error> {
        self.get_wrapped_object().get_property(name)
    }

    fn define_getter(&self, name: Value, value: Value) -> Result<(), Error> {
        self.get_wrapped_object().define_getter(name, value)
    }
    fn define_setter(&self, name: Value, value: Value) -> Result<(), Error> {
        self.get_wrapped_object().define_setter(name, value)
    }
    fn delete_property(&self, name: &Value) -> Result<Option<Value>, Error> {
        self.get_wrapped_object().delete_property(name)
    }

    fn contains_key(&self, name: &Value) -> Result<bool, Error> {
        self.get_wrapped_object().contains_key(name)
    }

    fn has_key(&self, name: &Value) -> Result<bool, Error> {
        self.get_wrapped_object().has_key(name)
    }

    fn name(&self) -> String {
        self.get_wrapped_object().name()
    }

    fn to_string(&self, realm: &mut Realm) -> Result<YSString, Error> {
        self.get_wrapped_object().to_string(realm)
    }
    fn to_string_internal(&self) -> Result<YSString, Error> {
        self.get_wrapped_object().to_string_internal()
    }

    #[allow(clippy::type_complexity)]
    fn properties(&self) -> Result<Vec<(Value, Value)>, Error> {
        self.get_wrapped_object().properties()
    }

    fn keys(&self) -> Result<Vec<Value>, Error> {
        self.get_wrapped_object().keys()
    }

    fn values(&self) -> Result<Vec<Value>, Error> {
        self.get_wrapped_object().values()
    }

    fn into_object(self) -> Object
    where
        Self: Sized + 'static,
    {
        let boxed: Box<dyn Obj> = Box::new(self);

        Object::from_boxed(boxed)
    }

    fn into_value(self) -> Value
    where
        Self: Sized + 'static,
    {
        Value::Object(self.into_object())
    }

    fn get_array_or_done(&self, index: usize) -> Result<(bool, Option<Value>), Error> {
        self.get_wrapped_object().get_array_or_done(index)
    }

    fn clear_values(&self) -> Result<(), Error> {
        self.get_wrapped_object().clear_values()
    }

    fn call(
        &self,
        realm: &mut Realm,
        args: Vec<Value>,
        this: Value,
    ) -> Result<Value, Error> {
        self.get_wrapped_object().call(realm, args, this)
    }

    fn is_function(&self) -> bool {
        self.get_wrapped_object().is_function()
    }
    fn primitive(&self) -> Option<Value> {
        self.get_wrapped_object().primitive()
    }

    fn prototype(&self) -> Result<ObjectProperty, Error> {
        self.get_wrapped_object().prototype()
    }

    fn set_prototype(&self, proto: ObjectProperty) -> Result<(), Error> {
        self.get_wrapped_object().set_prototype(proto)
    }

    fn constructor(&self) -> Result<ObjectProperty, Error> {
        self.get_wrapped_object().constructor()
    }

    /// # Safety
    /// This function should only return references that are actually in the object!
    /// Else it will leak memory and cause undefined behavior, same for references that are in the object but not known to the gc!
    unsafe fn custom_gc_refs(&self) -> Vec<GcRef<BoxedObj>> {
        self.get_wrapped_object().custom_gc_refs()
    }

    fn class_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> Result<Value, Error> {
        self.get_wrapped_object().construct(realm, args)
    }

    fn is_constructor(&self) -> bool {
        self.get_wrapped_object().is_constructor()
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

impl<T: ObjectImpl> Obj for T {
    fn define_property(&self, name: Value, value: Value) -> Result<(), Error> {
        ObjectImpl::define_property(self, name, value)
    }

    fn define_variable(&self, name: Value, value: Variable) -> Result<(), Error> {
        ObjectImpl::define_variable(self, name, value)
    }

    fn resolve_property(&self, name: &Value) -> Result<Option<ObjectProperty>, Error> {
        ObjectImpl::resolve_property(self, name)
    }

    fn get_property(&self, name: &Value) -> Result<Option<ObjectProperty>, Error> {
        ObjectImpl::get_property(self, name)
    }

    fn define_getter(&self, name: Value, value: Value) -> Result<(), Error> {
        ObjectImpl::define_getter(self, name, value)
    }

    fn define_setter(&self, name: Value, value: Value) -> Result<(), Error> {
        ObjectImpl::define_setter(self, name, value)
    }

    fn delete_property(&self, name: &Value) -> Result<Option<Value>, Error> {
        ObjectImpl::delete_property(self, name)
    }

    fn contains_key(&self, name: &Value) -> Result<bool, Error> {
        ObjectImpl::contains_key(self, name)
    }

    fn has_key(&self, name: &Value) -> Result<bool, Error> {
        ObjectImpl::has_key(self, name)
    }

    fn name(&self) -> String {
        ObjectImpl::name(self)
    }

    fn to_string(&self, realm: &mut Realm) -> Result<YSString, Error> {
        ObjectImpl::to_string(self, realm)
    }

    fn to_string_internal(&self) -> Result<YSString, Error> {
        ObjectImpl::to_string_internal(self)
    }

    fn properties(&self) -> Result<Vec<(Value, Value)>, Error> {
        ObjectImpl::properties(self)
    }

    fn keys(&self) -> Result<Vec<Value>, Error> {
        ObjectImpl::keys(self)
    }

    fn values(&self) -> Result<Vec<Value>, Error> {
        ObjectImpl::values(self)
    }

    fn into_object(self) -> Object
    where
        Self: Sized + 'static,
    {
        ObjectImpl::into_object(self)
    }

    fn into_value(self) -> Value
    where
        Self: Sized + 'static,
    {
        ObjectImpl::into_value(self)
    }

    fn get_array_or_done(&self, index: usize) -> Result<(bool, Option<Value>), Error> {
        ObjectImpl::get_array_or_done(self, index)
    }

    fn clear_values(&self) -> Result<(), Error> {
        ObjectImpl::clear_values(self)
    }

    fn call(
        &self,
        realm: &mut Realm,
        args: Vec<Value>,
        this: Value,
    ) -> Result<Value, Error> {
        ObjectImpl::call(self, realm, args, this)
    }

    fn is_function(&self) -> bool {
        ObjectImpl::is_function(self)
    }

    fn primitive(&self) -> Option<Value> {
        ObjectImpl::primitive(self)
    }

    fn prototype(&self) -> Result<ObjectProperty, Error> {
        ObjectImpl::prototype(self)
    }

    fn set_prototype(&self, proto: ObjectProperty) -> Result<(), Error> {
        ObjectImpl::set_prototype(self, proto)
    }

    fn constructor(&self) -> Result<ObjectProperty, Error> {
        ObjectImpl::constructor(self)
    }

    unsafe fn custom_gc_refs(&self) -> Vec<GcRef<BoxedObj>> {
        ObjectImpl::custom_gc_refs(self)
    }

    fn class_name(&self) -> &'static str {
        ObjectImpl::class_name(self)
    }

    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> Result<Value, Error> {
        ObjectImpl::construct(self, realm, args)
    }

    fn is_constructor(&self) -> bool {
        ObjectImpl::is_constructor(self)
    }

    unsafe fn inner_downcast(&self, ty: TypeId) -> Option<NonNull<()>> {
        ObjectImpl::inner_downcast(self, ty)
    }
}
