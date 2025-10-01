pub use super::object_impl::*;
use super::{Attributes, IntoValue, IntoValueRef, Value, Variable};
use crate::error::Error;
use crate::{Realm, Res, Symbol, ValueResult};
use indexmap::Equivalent;
use std::any::{Any, TypeId};
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
#[cfg(feature = "dbg_object_gc")]
use std::sync::atomic::AtomicIsize;
use yavashark_garbage::{Collectable, Gc, GcRef, OwningGcGuard, Weak};
use yavashark_string::{ToYSString, YSString};

pub trait AsAny {
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn as_any(&self) -> &dyn Any;
}

impl<T: Sized + 'static> AsAny for T {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub trait Obj: Debug + AsAny + Any + 'static {
    fn define_property(&self, name: Value, value: Value) -> Res;

    fn define_variable(&self, name: Value, value: Variable) -> Res;

    fn resolve_property(&self, name: &Value) -> Res<Option<ObjectProperty>>;

    fn get_property(&self, name: &Value) -> Res<Option<ObjectProperty>>;

    fn define_getter(&self, name: Value, value: Value) -> Res;
    fn define_setter(&self, name: Value, value: Value) -> Res;
    fn delete_property(&self, name: &Value) -> Res<Option<Value>>;

    fn contains_key(&self, name: &Value) -> Res<bool> {
        Ok(self.get_property(name)?.is_some())
    }

    fn has_key(&self, name: &Value) -> Res<bool> {
        Ok(self.resolve_property(name)?.is_some())
    }

    fn name(&self) -> String;

    fn to_string(&self, realm: &mut Realm) -> Res<YSString>;
    fn to_string_internal(&self) -> Res<YSString>;

    #[allow(clippy::type_complexity)]
    fn properties(&self) -> Res<Vec<(Value, Value)>>;

    fn keys(&self) -> Res<Vec<Value>>;

    fn values(&self) -> Res<Vec<Value>>;

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

    fn get_array_or_done(&self, index: usize) -> Res<(bool, Option<Value>)>;

    fn clear_values(&self) -> Res;

    #[allow(unused_variables)]
    fn call(&self, realm: &mut Realm, args: Vec<Value>, this: Value) -> ValueResult {
        Err(Error::ty_error(format!(
            "{} is not a function",
            self.name()
        )))
    }

    fn is_function(&self) -> bool {
        false
    }

    fn primitive(&self) -> Option<Value> {
        None
    }

    fn prototype(&self) -> Res<ObjectProperty> {
        Ok(self
            .resolve_property(&"__proto__".into())?
            .unwrap_or(Value::Undefined.into()))
    }

    fn set_prototype(&self, proto: ObjectProperty) -> Res {
        self.define_property("__proto__".into(), proto.value)
    }

    fn constructor(&self) -> Res<ObjectProperty> {
        Ok(self
            .resolve_property(&"constructor".into())?
            .unwrap_or(Value::Undefined.into()))
    }

    /// # Safety
    /// This function should only return references that are actually in the object!
    /// Else it will leak memory and cause undefined behavior, same for references that are in the object but not known to the gc!
    unsafe fn custom_gc_refs(&self) -> Vec<GcRef<BoxedObj>> {
        Vec::new()
    }

    fn class_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    #[allow(unused_variables)]
    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> ValueResult {
        Err(Error::ty_error(format!(
            "{} is not a constructor",
            self.name()
        )))
    }

    fn is_constructor(&self) -> bool {
        false
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

pub trait MutObj: Debug + AsAny + 'static {
    fn define_property(&mut self, name: Value, value: Value) -> Res;

    fn define_variable(&mut self, name: Value, value: Variable) -> Res;

    fn resolve_property(&self, name: &Value) -> Res<Option<ObjectProperty>>;

    fn get_property(&self, name: &Value) -> Res<Option<ObjectProperty>>;

    fn define_getter(&mut self, name: Value, value: Value) -> Res;
    fn define_setter(&mut self, name: Value, value: Value) -> Res;
    fn delete_property(&mut self, name: &Value) -> Res<Option<Value>>;

    fn contains_key(&self, name: &Value) -> Res<bool, Error> {
        Ok(self.get_property(name)?.is_some())
    }

    fn has_key(&self, name: &Value) -> Res<bool> {
        Ok(self.resolve_property(name)?.is_some())
    }

    fn name(&self) -> String;

    fn to_string(&self, realm: &mut Realm) -> Res<YSString>;
    fn to_string_internal(&self) -> Res<YSString>;

    #[allow(clippy::type_complexity)]
    fn properties(&self) -> Res<Vec<(Value, Value)>>;

    fn keys(&self) -> Res<Vec<Value>>;

    fn values(&self) -> Res<Vec<Value>>;

    fn get_array_or_done(&self, index: usize) -> Res<(bool, Option<Value>)>;

    fn clear_values(&mut self) -> Res;

    #[allow(unused_variables)]
    fn call(&mut self, realm: &mut Realm, args: Vec<Value>, this: Value) -> ValueResult {
        Err(Error::ty_error(format!(
            "{} is not a function",
            self.name()
        )))
    }

    fn is_function(&self) -> bool {
        false
    }

    fn primitive(&self) -> Option<Value> {
        None
    }

    fn prototype(&self) -> Res<ObjectProperty> {
        Ok(self
            .resolve_property(&"__proto__".into())?
            .unwrap_or(Value::Undefined.into()))
    }

    fn set_prototype(&mut self, proto: ObjectProperty) -> Res {
        self.define_property("__proto__".into(), proto.value)
    }

    fn constructor(&self) -> Res<ObjectProperty> {
        Ok(self
            .resolve_property(&"constructor".into())?
            .unwrap_or(Value::Undefined.into()))
    }

    /// # Safety
    /// This function should only return references that are actually in the object!
    /// Else it will leak memory and cause undefined behavior, same for references that are in the object but not known to the gc!
    unsafe fn custom_gc_refs(&self) -> Vec<GcRef<BoxedObj>> {
        Vec::new()
    }

    fn class_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn construct(&mut self, _realm: &mut Realm, _args: Vec<Value>) -> ValueResult {
        Err(Error::ty_error(format!(
            "{} is not a constructor",
            self.name()
        )))
    }

    fn is_constructor(&self) -> bool {
        false
    }
}
#[cfg(feature = "dbg_object_gc")]
pub struct ObjectCount(AtomicIsize);

#[cfg(feature = "dbg_object_gc")]
impl ObjectCount {
    const fn new() -> Self {
        Self(AtomicIsize::new(0))
    }

    fn increment(&self) {
        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    fn decrement(&self) {
        self.0.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn get(&self) -> isize {
        self.0.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[cfg(feature = "dbg_object_gc")]
pub static OBJECT_COUNT: ObjectCount = ObjectCount::new();
#[cfg(feature = "dbg_object_gc")]
pub static OBJECT_ALLOC: ObjectCount = ObjectCount::new();

#[repr(transparent)]
pub struct BoxedObj(Box<dyn Obj>);

impl Deref for BoxedObj {
    type Target = dyn Obj;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl DerefMut for BoxedObj {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

unsafe impl Collectable for BoxedObj {
    fn get_refs(&self) -> Vec<GcRef<Self>> {
        let mut refs = Vec::new();

        self.0
            .properties()
            .unwrap_or_default()
            .into_iter()
            .for_each(|(n, v)| {
                if let Value::Object(o) = n {
                    refs.push(o.0.get_ref());
                }

                if let Value::Object(o) = v {
                    refs.push(o.0.get_ref());
                }
            });

        if let Ok(p) = self.0.prototype() {
            if let Value::Object(o) = p.value {
                refs.push(o.0.get_ref());
            }

            if let Value::Object(o) = p.get {
                refs.push(o.0.get_ref());
            }

            if let Value::Object(o) = p.set {
                refs.push(o.0.get_ref());
            }
        }

        unsafe {
            // Safety: unsafe is only for the implementer, not for us - we are safe
            refs.append(&mut self.0.custom_gc_refs());
        }

        refs
    }

    #[cfg(any(feature = "obj_dbg", feature = "obj_trace"))]
    fn trace_name(&self) -> &'static str {
        self.0.class_name()
    }
}

impl BoxedObj {
    fn new(obj: Box<dyn Obj>) -> Self {
        #[cfg(feature = "dbg_object_gc")]
        {
            OBJECT_COUNT.increment();
            OBJECT_ALLOC.increment();
        }
        Self(obj)
    }

    #[allow(clippy::needless_lifetimes)]
    pub fn downcast<'a, T: 'static>(&'a self) -> Option<&'a T> {
        // Safety:
        // - we only interpret the returned pointer as T
        // - we only say the reference is valid for 'a this being the lifetime of self
        unsafe {
            let ptr = self.deref().inner_downcast(TypeId::of::<T>())?.cast();

            Some(ptr.as_ref())
        }
    }
}

#[derive(Clone)]
pub struct Object(Gc<BoxedObj>);

impl Deref for Object {
    type Target = Gc<BoxedObj>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Object {
    /// This function shouldn't be used in production code, only for debugging
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.to_string_internal() {
            Ok(s) => write!(f, "{s}"),
            Err(_) => write!(f, "Error: error while converting object to string"),
        }
    }
}

impl ToYSString for Object {
    fn to_ys_string(&self) -> YSString {
        self.to_string_internal().unwrap_or_else(|_| {
            YSString::new_static("Error: error while converting object to string")
        })
    }
}

#[cfg(feature = "dbg_object_gc")]
impl Drop for BoxedObj {
    fn drop(&mut self) {
        OBJECT_COUNT.decrement();
    }
}

impl Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", *self)
    }
}

impl Hash for Object {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_ptr().hash(state);
    }
}

impl Eq for Object {}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Object {
    pub fn resolve_property(&self, name: &Value, realm: &mut Realm) -> Res<Option<Value>> {
        let Some(p) = self.0.resolve_property(name)? else {
            return Ok(None);
        };

        p.get(Value::Object(self.clone()), realm).map(Some)
    }

    pub fn call_method(&self, name: &Value, realm: &mut Realm, args: Vec<Value>) -> ValueResult {
        let method = self.resolve_property(name, realm)?.ok_or_else(|| {
            let name = match name.to_string(realm) {
                Ok(name) => name,
                Err(e) => return e,
            };

            Error::reference_error(format!(
                "Cannot call {} on {}",
                name,
                self.to_string_internal().unwrap_or_default()
            ))
        })?;

        method.call(realm, args, self.clone().into())
    }
    pub fn resolve_property_no_get_set(&self, name: &Value) -> Res<Option<ObjectProperty>> {
        self.0.resolve_property(name)
    }

    pub fn get_property(&self, name: &Value) -> Res<ObjectProperty> {
        self.0
            .get_property(name)?
            .ok_or(Error::reference_error(format!(
                "{name} does not exist on object"
            )))
    }

    pub fn get_property_opt(&self, name: &Value) -> Res<Option<ObjectProperty>> {
        self.0.get_property(name)
    }

    #[must_use]
    pub fn name(&self) -> String {
        self.0.name()
    }

    #[must_use]
    pub fn custom_refs(&self) -> Vec<GcRef<BoxedObj>> {
        // Safety: unsafe is only for the implementer, not for us - we are safe
        unsafe { self.custom_gc_refs() }
    }

    #[must_use]
    pub fn id(&self) -> usize {
        self.0.ptr_id()
    }

    pub fn downcast<T: 'static>(&self) -> Option<OwningGcGuard<'static, BoxedObj, T>> {
        self.get_owning().maybe_map(BoxedObj::downcast::<T>).ok()
    }

    pub fn set(
        &self,
        name: impl Into<Value>,
        value: impl Into<Variable>,
        _realm: &mut Realm,
    ) -> ValueResult {
        let name = name.into();
        let value = value.into();

        self.0
            .define_variable(name, value)
            .map(|()| Value::Undefined)
    }

    pub fn get(&self, name: impl IntoValueRef, realm: &mut Realm) -> ValueResult {
        let name = name.into_value_ref();

        self.0
            .resolve_property(name.as_ref())?
            .map_or(Ok(Value::Undefined), |x| {
                x.get(Value::Object(self.clone()), realm)
            })
    }

    pub fn get_opt(&self, name: impl IntoValueRef, realm: &mut Realm) -> Res<Option<Value>> {
        let name = name.into_value_ref();

        self.0
            .resolve_property(name.as_ref())?
            .map_or(Ok(None), |x| {
                Ok(Some(x.get(Value::Object(self.clone()), realm)?))
            })
    }

    pub fn to_primitive(&self, hint: Hint, realm: &mut Realm) -> ValueResult {
        if let Some(prim) = self.primitive() {
            return prim.assert_no_object();
        }

        let to_prim = self.resolve_property(&Symbol::TO_PRIMITIVE.into(), realm)?;

        match to_prim {
            Some(Value::Object(to_prim)) => {
                if to_prim.is_function() {
                    return to_prim
                        .call(realm, vec![hint.into_value()], self.clone().into())?
                        .assert_no_object();
                }
            }
            Some(to_prim) => return Ok(to_prim),
            None => {}
        }

        if hint == Hint::String {
            let to_string = self.resolve_property(&"toString".into(), realm)?;

            if let Some(Value::Object(to_string)) = to_string {
                if to_string.is_function() {
                    return to_string
                        .call(realm, Vec::new(), self.clone().into())?
                        .assert_no_object();
                }
            }

            let to_value = self.resolve_property(&"valueOf".into(), realm)?;

            if let Some(Value::Object(to_value)) = to_value {
                if to_value.is_function() {
                    return to_value
                        .call(realm, Vec::new(), self.clone().into())?
                        .assert_no_object();
                }
            }
        }

        let to_value = self.resolve_property(&"valueOf".into(), realm)?;

        if let Some(Value::Object(to_value)) = to_value {
            if to_value.is_function() {
                let val = to_value.call(realm, Vec::new(), self.clone().into())?;

                if !val.is_object() {
                    return Ok(val);
                }
            }
        }

        let to_string = self.resolve_property(&"toString".into(), realm)?;

        if let Some(Value::Object(to_string)) = to_string {
            if to_string.is_function() {
                return to_string
                    .call(realm, Vec::new(), self.clone().into())?
                    .assert_no_object();
            }
        }
        Err(Error::ty("Cannot convert object to primitive"))
    }

    pub fn enum_properties(&self) -> Res<Vec<(Value, ObjectProperty)>> {
        let mut properties = Vec::new();

        for name in self.0.keys()? {
            if let Some(prop) = self.get_property_opt(&name)? {
                if prop.attributes.is_enumerable() {
                    properties.push((name, prop));
                }
            }
        }

        Ok(properties)
    }

    #[must_use]
    pub fn downgrade(&self) -> WeakObject {
        WeakObject::new(self)
    }

    #[must_use]
    pub fn gc_ref(&self) -> Option<GcRef<BoxedObj>> {
        Some(self.get_ref())
    }
}

impl From<Box<dyn Obj>> for Object {
    fn from(obj: Box<dyn Obj>) -> Self {
        Self(Gc::new(BoxedObj::new(obj)))
    }
}

impl From<Gc<BoxedObj>> for Object {
    fn from(obj: Gc<BoxedObj>) -> Self {
        Self(obj)
    }
}

impl Object {
    #[must_use]
    pub fn from_boxed(obj: Box<dyn Obj>) -> Self {
        Self(Gc::new(BoxedObj::new(obj)))
    }

    pub fn new<O: Obj + 'static>(obj: O) -> Self {
        Self(Gc::new(BoxedObj::new(Box::new(obj))))
    }

    pub fn to_string(&self, realm: &mut Realm) -> Res<YSString> {
        self.0.to_string(realm)
    }
}

#[derive(Clone)]
pub struct WeakObject(Weak<BoxedObj>);

impl WeakObject {
    #[must_use]
    pub fn new(obj: &Object) -> Self {
        Self(Gc::downgrade(&obj.0))
    }

    pub fn upgrade(&self) -> Option<Object> {
        self.0.upgrade().map(Object::from)
    }
}

impl Debug for WeakObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.upgrade() {
            Some(obj) => write!(f, "WeakObject({obj})"),
            None => write!(f, "WeakObject(<dead>)"),
        }
    }
}

impl PartialEq for WeakObject {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Equivalent<Object> for WeakObject {
    fn equivalent(&self, key: &Object) -> bool {
        self.0 == key.0
    }
}

impl Equivalent<WeakObject> for Object {
    fn equivalent(&self, key: &WeakObject) -> bool {
        self.0 == key.0
    }
}

impl Hash for WeakObject {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_ptr().hash(state);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hint {
    Number,
    String,
    None,
}

impl IntoValue for Hint {
    fn into_value(self) -> Value {
        match self {
            Self::Number => "number".into(),
            Self::String => "string".into(),
            Self::None => Value::Undefined,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ObjectProperty {
    pub value: Value,
    pub attributes: Attributes,
    pub get: Value,
    pub set: Value,
}

impl ObjectProperty {
    #[must_use]
    pub const fn new(value: Value) -> Self {
        Self {
            value,
            attributes: Attributes::new(),
            get: Value::Undefined,
            set: Value::Undefined,
        }
    }

    #[must_use]
    pub const fn getter(value: Value) -> Self {
        Self {
            value: Value::Undefined,
            attributes: Attributes::config(),
            get: value,
            set: Value::Undefined,
        }
    }

    #[must_use]
    pub const fn setter(value: Value) -> Self {
        Self {
            value: Value::Undefined,
            attributes: Attributes::config(),
            get: Value::Undefined,
            set: value,
        }
    }

    pub fn get(self, this: Value, realm: &mut Realm) -> ValueResult {
        if self.get.is_nullish() {
            Ok(self.value)
        } else {
            self.get.call(realm, vec![], this)
        }
    }

    pub fn resolve(&self, this: Value, realm: &mut Realm) -> ValueResult {
        if self.get.is_nullish() {
            Ok(self.value.copy())
        } else {
            self.get.call(realm, vec![], this)
        }
    }

    #[must_use]
    pub fn copy(&self) -> Self {
        Self {
            value: self.value.copy(),
            attributes: self.attributes,
            get: self.get.copy(),
            set: self.set.copy(),
        }
    }

    pub fn descriptor(self, obj: &Object) -> Res {
        if !self.set.is_undefined() || !self.get.is_undefined() {
            obj.define_property("get".into(), self.get)?;
            obj.define_property("set".into(), self.set)?;
        } else {
            obj.define_property("value".into(), self.value)?;
        }

        obj.define_property("writable".into(), self.attributes.is_writable().into())?;
        obj.define_property("enumerable".into(), self.attributes.is_enumerable().into())?;
        obj.define_property(
            "configurable".into(),
            self.attributes.is_configurable().into(),
        )?;

        Ok(())
    }
}

impl From<Variable> for ObjectProperty {
    fn from(v: Variable) -> Self {
        Self {
            value: v.value,
            attributes: v.properties,
            get: Value::Undefined,
            set: Value::Undefined,
        }
    }
}

// impl<C: Ctx> From<Value> for ObjectProperty {
//     fn from(v: Value) -> Self {
//         Self {
//             value: v,
//             attributes: Attributes::new(),
//             get: Value::Undefined,
//             set: Value::Undefined,
//         }
//     }
// }

impl<V: Into<Value>> From<V> for ObjectProperty {
    fn from(v: V) -> Self {
        Self {
            value: v.into(),
            attributes: Attributes::new(),
            get: Value::Undefined,
            set: Value::Undefined,
        }
    }
}
