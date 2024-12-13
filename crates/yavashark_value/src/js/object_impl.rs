use crate::{AsAny, BoxedObj, Error, Obj, Object, ObjectProperty, Realm, Value, Variable};
use std::cell::RefCell;
use std::fmt::Debug;
use yavashark_garbage::GcRef;

pub trait ObjectImpl<R: Realm>: Debug + AsAny {
    /// the returned object should NOT be a reference to self, but a reference to the object that is wrapped by self
    fn get_wrapped_object(&self) -> &impl Obj<R>;
    fn get_wrapped_object_mut(&mut self) -> &mut impl Obj<R>;

    fn define_property(&mut self, name: Value<R>, value: Value<R>) {
        self.get_wrapped_object_mut().define_property(name, value);
    }

    fn define_variable(&mut self, name: Value<R>, value: Variable<R>) {
        self.get_wrapped_object_mut().define_variable(name, value);
    }

    fn resolve_property(&self, name: &Value<R>) -> Option<ObjectProperty<R>> {
        self.get_wrapped_object().resolve_property(name)
    }

    fn get_property(&self, name: &Value<R>) -> Option<&Value<R>> {
        self.get_wrapped_object().get_property(name)
    }

    fn define_getter(&mut self, name: Value<R>, value: Value<R>) -> Result<(), Error<R>> {
        self.get_wrapped_object_mut().define_getter(name, value)
    }
    fn define_setter(&mut self, name: Value<R>, value: Value<R>) -> Result<(), Error<R>> {
        self.get_wrapped_object_mut().define_setter(name, value)
    }
    fn get_getter(&self, name: &Value<R>) -> Option<Value<R>> {
        self.get_wrapped_object().get_getter(name)
    }
    fn get_setter(&self, name: &Value<R>) -> Option<Value<R>> {
        self.get_wrapped_object().get_setter(name)
    }

    fn delete_property(&mut self, name: &Value<R>) -> Option<Value<R>> {
        self.get_wrapped_object_mut().delete_property(name)
    }

    fn contains_key(&self, name: &Value<R>) -> bool {
        self.get_wrapped_object().contains_key(name)
    }

    fn name(&self) -> String {
        self.get_wrapped_object().name()
    }

    fn to_string(&self, realm: &mut R) -> Result<String, Error<R>> {
        self.get_wrapped_object().to_string(realm)
    }
    fn to_string_internal(&self) -> String {
        self.get_wrapped_object().to_string_internal()
    }

    fn properties(&self) -> Vec<(Value<R>, Value<R>)> {
        self.get_wrapped_object().properties()
    }

    fn keys(&self) -> Vec<Value<R>> {
        self.get_wrapped_object().keys()
    }

    fn values(&self) -> Vec<Value<R>> {
        self.get_wrapped_object().values()
    }

    fn into_object(self) -> Object<R>
    where
        Self: Sized + 'static,
    {
        let boxed: Box<dyn Obj<R>> = Box::new(self);

        Object::from_boxed(boxed)
    }

    fn into_value(self) -> Value<R>
    where
        Self: Sized + 'static,
    {
        Value::Object(self.into_object())
    }

    fn get_array_or_done(&self, index: usize) -> (bool, Option<Value<R>>) {
        self.get_wrapped_object().get_array_or_done(index)
    }

    fn clear_values(&mut self) {
        self.get_wrapped_object_mut().clear_values()
    }

    fn call(
        &mut self,
        realm: &mut R,
        args: Vec<Value<R>>,
        this: Value<R>,
    ) -> Result<Value<R>, Error<R>> {
        self.get_wrapped_object_mut().call(realm, args, this)
    }

    fn is_function(&self) -> bool {
        self.get_wrapped_object().is_function()
    }

    fn prototype(&self) -> ObjectProperty<R> {
        self.get_wrapped_object().prototype()
    }

    fn constructor(&self) -> ObjectProperty<R> {
        self.get_wrapped_object().constructor()
    }

    /// # Safety
    /// This function should only return references that are actually in the object!
    /// Else it will leak memory and cause undefined behavior, same for references that are in the object but not known to the gc!
    unsafe fn custom_gc_refs(&self) -> Vec<GcRef<RefCell<BoxedObj<R>>>> {
        self.get_wrapped_object().custom_gc_refs()
    }

    fn class_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn get_constructor_value(&self, _realm: &mut R) -> Option<Value<R>> {
        None
    }

    fn get_constructor_proto(&self, _realm: &mut R) -> Option<Value<R>> {
        None
    }

    fn special_constructor(&self) -> bool {
        false
    }
}

impl<T: ObjectImpl<R>, R: Realm> Obj<R> for T {
    fn define_property(&mut self, name: Value<R>, value: Value<R>) {
        ObjectImpl::define_property(self, name, value);
    }

    fn define_variable(&mut self, name: Value<R>, value: Variable<R>) {
        ObjectImpl::define_variable(self, name, value);
    }

    fn resolve_property(&self, name: &Value<R>) -> Option<ObjectProperty<R>> {
        ObjectImpl::resolve_property(self, name)
    }

    fn get_property(&self, name: &Value<R>) -> Option<&Value<R>> {
        ObjectImpl::get_property(self, name)
    }

    fn define_getter(&mut self, name: Value<R>, value: Value<R>) -> Result<(), Error<R>> {
        ObjectImpl::define_getter(self, name, value)
    }

    fn define_setter(&mut self, name: Value<R>, value: Value<R>) -> Result<(), Error<R>> {
        ObjectImpl::define_setter(self, name, value)
    }

    fn get_getter(&self, name: &Value<R>) -> Option<Value<R>> {
        ObjectImpl::get_getter(self, name)
    }

    fn get_setter(&self, name: &Value<R>) -> Option<Value<R>> {
        ObjectImpl::get_setter(self, name)
    }

    fn delete_property(&mut self, name: &Value<R>) -> Option<Value<R>> {
        ObjectImpl::delete_property(self, name)
    }

    fn contains_key(&self, name: &Value<R>) -> bool {
        ObjectImpl::contains_key(self, name)
    }

    fn name(&self) -> String {
        ObjectImpl::name(self)
    }

    fn to_string(&self, realm: &mut R) -> Result<String, Error<R>> {
        ObjectImpl::to_string(self, realm)
    }

    fn to_string_internal(&self) -> String {
        ObjectImpl::to_string_internal(self)
    }

    fn properties(&self) -> Vec<(Value<R>, Value<R>)> {
        ObjectImpl::properties(self)
    }

    fn keys(&self) -> Vec<Value<R>> {
        ObjectImpl::keys(self)
    }

    fn values(&self) -> Vec<Value<R>> {
        ObjectImpl::values(self)
    }

    fn into_object(self) -> Object<R>
    where
        Self: Sized + 'static,
    {
        ObjectImpl::into_object(self)
    }

    fn into_value(self) -> Value<R>
    where
        Self: Sized + 'static,
    {
        ObjectImpl::into_value(self)
    }

    fn get_array_or_done(&self, index: usize) -> (bool, Option<Value<R>>) {
        ObjectImpl::get_array_or_done(self, index)
    }

    fn clear_values(&mut self) {
        ObjectImpl::clear_values(self);
    }

    fn call(
        &mut self,
        realm: &mut R,
        args: Vec<Value<R>>,
        this: Value<R>,
    ) -> Result<Value<R>, Error<R>> {
        ObjectImpl::call(self, realm, args, this)
    }

    fn is_function(&self) -> bool {
        ObjectImpl::is_function(self)
    }

    fn prototype(&self) -> ObjectProperty<R> {
        ObjectImpl::prototype(self)
    }

    fn constructor(&self) -> ObjectProperty<R> {
        ObjectImpl::constructor(self)
    }

    unsafe fn custom_gc_refs(&self) -> Vec<GcRef<RefCell<BoxedObj<R>>>> {
        ObjectImpl::custom_gc_refs(self)
    }

    fn class_name(&self) -> &'static str {
        ObjectImpl::class_name(self)
    }

    fn get_constructor_value(&self, realm: &mut R) -> Option<Value<R>> {
        ObjectImpl::get_constructor_value(self, realm)
    }

    fn get_constructor_proto(&self, realm: &mut R) -> Option<Value<R>> {
        ObjectImpl::get_constructor_proto(self, realm)
    }

    fn special_constructor(&self) -> bool {
        ObjectImpl::special_constructor(self)
    }
}
