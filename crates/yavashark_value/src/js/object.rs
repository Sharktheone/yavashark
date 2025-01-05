use std::any::Any;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
#[cfg(feature = "dbg_object_gc")]
use std::sync::atomic::AtomicIsize;

use yavashark_garbage::{Collectable, Gc, GcRef};

use crate::js::context::Realm;
use crate::variable::Variable;
use crate::{Attributes, Error};

use super::Value;

pub use super::object_impl::*;

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

pub trait Obj<R: Realm>: Debug + AsAny + 'static {
    fn define_property(&self, name: Value<R>, value: Value<R>) -> Result<(), Error<R>>;

    fn define_variable(&self, name: Value<R>, value: Variable<R>) -> Result<(), Error<R>>;

    fn resolve_property(&self, name: &Value<R>) -> Result<Option<ObjectProperty<R>>, Error<R>>;

    fn get_property(&self, name: &Value<R>) -> Result<Option<Value<R>>, Error<R>>;

    fn define_getter(&self, name: Value<R>, value: Value<R>) -> Result<(), Error<R>>;
    fn define_setter(&self, name: Value<R>, value: Value<R>) -> Result<(), Error<R>>;
    fn get_getter(&self, name: &Value<R>) -> Result<Option<Value<R>>, Error<R>>;
    fn get_setter(&self, name: &Value<R>) -> Result<Option<Value<R>>, Error<R>>;

    fn delete_property(&self, name: &Value<R>) -> Result<Option<Value<R>>, Error<R>>;

    fn contains_key(&self, name: &Value<R>) -> Result<bool, Error<R>> {
        Ok(self.get_property(name)?.is_some())
    }

    fn name(&self) -> String;

    fn to_string(&self, realm: &mut R) -> Result<String, Error<R>>;
    fn to_string_internal(&self) -> Result<String, Error<R>>;

    fn properties(&self) -> Result<Vec<(Value<R>, Value<R>)>, Error<R>>;

    fn keys(&self) -> Result<Vec<Value<R>>, Error<R>>;

    fn values(&self) -> Result<Vec<Value<R>>, Error<R>>;

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

    fn get_array_or_done(&self, index: usize) -> Result<(bool, Option<Value<R>>), Error<R>>;

    fn clear_values(&self) -> Result<(), Error<R>>;

    #[allow(unused_variables)]
    fn call(
        &self,
        realm: &mut R,
        args: Vec<Value<R>>,
        this: Value<R>,
    ) -> Result<Value<R>, Error<R>> {
        Err(Error::ty_error(format!(
            "{} is not a function",
            self.name()
        )))
    }

    fn is_function(&self) -> bool {
        false
    }

    fn prototype(&self) -> Result<ObjectProperty<R>, Error<R>> {
        Ok(self
            .resolve_property(&"__proto__".into())?
            .unwrap_or(Value::Undefined.into()))
    }

    fn constructor(&self) -> Result<ObjectProperty<R>, Error<R>> {
        Ok(self
            .resolve_property(&"constructor".into())?
            .unwrap_or(Value::Undefined.into()))
    }

    /// # Safety
    /// This function should only return references that are actually in the object!
    /// Else it will leak memory and cause undefined behavior, same for references that are in the object but not known to the gc!
    unsafe fn custom_gc_refs(&self) -> Vec<GcRef<BoxedObj<R>>> {
        Vec::new()
    }

    fn class_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn get_constructor_value(&self, _realm: &mut R) -> Result<Option<Value<R>>, Error<R>> {
        Ok(None)
    }

    fn get_constructor_proto(&self, _realm: &mut R) -> Result<Option<Value<R>>, Error<R>> {
        Ok(None)
    }

    fn special_constructor(&self) -> bool {
        false
    }
}

pub trait MutObj<R: Realm>: Debug + AsAny + 'static {
    fn define_property(&mut self, name: Value<R>, value: Value<R>) -> Result<(), Error<R>>;

    fn define_variable(&mut self, name: Value<R>, value: Variable<R>) -> Result<(), Error<R>>;

    fn resolve_property(&self, name: &Value<R>) -> Result<Option<&ObjectProperty<R>>, Error<R>>;

    fn get_property(&self, name: &Value<R>) -> Result<Option<&Value<R>>, Error<R>>;

    fn define_getter(&mut self, name: Value<R>, value: Value<R>) -> Result<(), Error<R>>;
    fn define_setter(&mut self, name: Value<R>, value: Value<R>) -> Result<(), Error<R>>;
    fn get_getter(&self, name: &Value<R>) -> Result<Option<Value<R>>, Error<R>>;
    fn get_setter(&self, name: &Value<R>) -> Result<Option<Value<R>>, Error<R>>;

    fn delete_property(&mut self, name: &Value<R>) -> Result<Option<Value<R>>, Error<R>>;

    fn contains_key(&self, name: &Value<R>) -> Result<bool, Error<R>> {
        Ok(self.get_property(name)?.is_some())
    }

    fn name(&self) -> String;

    fn to_string(&self, realm: &mut R) -> Result<String, Error<R>>;
    fn to_string_internal(&self) -> Result<String, Error<R>>;

    fn properties(&self) -> Result<Vec<(Value<R>, Value<R>)>, Error<R>>;

    fn keys(&self) -> Result<Vec<Value<R>>, Error<R>>;

    fn values(&self) -> Result<Vec<Value<R>>, Error<R>>;

    fn get_array_or_done(&self, index: usize) -> Result<(bool, Option<Value<R>>), Error<R>>;

    fn clear_values(&mut self) -> Result<(), Error<R>>;

    #[allow(unused_variables)]
    fn call(
        &mut self,
        realm: &mut R,
        args: Vec<Value<R>>,
        this: Value<R>,
    ) -> Result<Value<R>, Error<R>> {
        Err(Error::ty_error(format!(
            "{} is not a function",
            self.name()
        )))
    }

    fn is_function(&self) -> bool {
        false
    }

    fn prototype(&self) -> Result<ObjectProperty<R>, Error<R>> {
        Ok(self
            .resolve_property(&"__proto__".into())?
            .cloned()
            .unwrap_or(Value::Undefined.into()))
    }

    fn constructor(&self) -> Result<ObjectProperty<R>, Error<R>> {
        Ok(self
            .resolve_property(&"constructor".into())?
            .cloned()
            .unwrap_or(Value::Undefined.into()))
    }

    /// # Safety
    /// This function should only return references that are actually in the object!
    /// Else it will leak memory and cause undefined behavior, same for references that are in the object but not known to the gc!
    unsafe fn custom_gc_refs(&self) -> Vec<GcRef<BoxedObj<R>>> {
        Vec::new()
    }

    fn class_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn get_constructor_value(&self, _realm: &mut R) -> Result<Option<Value<R>>, Error<R>> {
        Ok(None)
    }

    fn get_constructor_proto(&self, _realm: &mut R) -> Result<Option<Value<R>>, Error<R>> {
        Ok(None)
    }

    fn special_constructor(&self) -> bool {
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
pub struct BoxedObj<C: Realm>(Box<dyn Obj<C>>);

impl<C: Realm> Deref for BoxedObj<C> {
    type Target = dyn Obj<C>;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<C: Realm> DerefMut for BoxedObj<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

unsafe impl<C: Realm> Collectable for BoxedObj<C> {
    fn get_refs(&self) -> Vec<GcRef<Self>> {
        let mut refs = Vec::new();

        self.0.properties().unwrap_or_default().into_iter().for_each(|(n, v)| {
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

impl<C: Realm> BoxedObj<C> {
    fn new(obj: Box<dyn Obj<C>>) -> Self {
        #[cfg(feature = "dbg_object_gc")]
        {
            OBJECT_COUNT.increment();
            OBJECT_ALLOC.increment();
        }
        Self(obj)
    }
}

#[derive(Clone)]
pub struct Object<C: Realm>(Gc<BoxedObj<C>>);

impl<C: Realm> Deref for Object<C> {
    type Target = Gc<BoxedObj<C>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(any(test, debug_assertions, feature = "display_object"))]
impl<C: Realm> Display for Object<C> {
    /// This function shouldn't be used in production code, only for debugging
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.to_string_internal() {
            Ok(s) => write!(f, "{}", s),
            Err(_) => write!(f, "Error: error while converting object to string"),
        }
    }
}

#[cfg(feature = "dbg_object_gc")]
impl<C: Realm> Drop for BoxedObj<C> {
    fn drop(&mut self) {
        OBJECT_COUNT.decrement();
    }
}

impl<C: Realm> Debug for Object<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", *self)
    }
}

impl<C: Realm> Hash for Object<C> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_ptr().hash(state);
    }
}

impl<C: Realm> Eq for Object<C> {}

impl<C: Realm> PartialEq for Object<C> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<C: Realm> Object<C> {
    pub fn resolve_property(
        &self,
        name: &Value<C>,
        realm: &mut C,
    ) -> Result<Option<Value<C>>, Error<C>> {
        let Some(p) = self.0.resolve_property(name)? else {
            return Ok(None);
        };

        p.get(Value::Object(self.clone()), realm).map(Some)
    }
    pub fn resolve_property_no_get_set(
        &self,
        name: &Value<C>,
    ) -> Result<Option<ObjectProperty<C>>, Error<C>> {
        self.0.resolve_property(name)
    }

    pub fn get_property(&self, name: &Value<C>) -> Result<Value<C>, Error<C>> {
        self.0
            .get_property(name)?
            .ok_or(Error::reference_error(format!(
                "{name} does not exist on object"
            )))
    }

    #[must_use]
    pub fn name(&self) -> String {
        self.0.name()
    }

    #[must_use]
    pub fn custom_refs(&self) -> Vec<GcRef<BoxedObj<C>>> {
        /// Safety: unsafe is only for the implementer, not for us - we are safe
        unsafe { self.custom_gc_refs() }
    }

    #[must_use]
    pub fn id(&self) -> usize {
        self.0.ptr_id()
    }

    // pub fn downcast<T: Obj<C> + 'static>(
    //     &self,
    // ) -> Result<Option<Gc<BoxedObj<C>, T>>, Error<C>> {
    //     let obj = self.get()?;
    //
    //     Ok(obj
    //         .maybe_map(|r| {
    //             let this = (**r).as_any();
    //
    //             this.downcast_ref::<T>()
    //         })
    //         .ok())
    // }
    //
    // pub fn downcast_mut<T: Obj<C> + 'static>(
    //     &self,
    // ) -> Result<Option<GcMutRefCellGuard<BoxedObj<C>, T>>, Error<C>> {
    //     let obj = self.get_mut()?;
    //
    //     Ok(obj
    //         .maybe_map(|r| {
    //             let this = (**r).as_any_mut();
    //
    //             this.downcast_mut::<T>()
    //         })
    //         .ok())
    // }
}

impl<C: Realm> From<Box<dyn Obj<C>>> for Object<C> {
    fn from(obj: Box<dyn Obj<C>>) -> Self {
        Self(Gc::new(BoxedObj::new(obj)))
    }
}

impl<C: Realm> Object<C> {
    #[must_use]
    pub fn from_boxed(obj: Box<dyn Obj<C>>) -> Self {
        Self(Gc::new((BoxedObj::new(obj))))
    }

    pub fn new<O: Obj<C> + 'static>(obj: O) -> Self {
        Self(Gc::new((BoxedObj::new(Box::new(obj)))))
    }

    pub fn to_string(&self, realm: &mut C) -> Result<String, Error<C>> {
        if let Some(to_string) = self.resolve_property(&"toString".into(), realm)? {
            let to_string = to_string.copy();

            let val = to_string.call(realm, vec![], Value::Object(self.clone()))?;
            
            val.to_string(realm)
        } else {
            self.0.to_string(realm)
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ObjectProperty<C: Realm> {
    pub value: Value<C>,
    pub attributes: Attributes,
    pub get: Value<C>,
    pub set: Value<C>,
}

impl<C: Realm> ObjectProperty<C> {
    #[must_use]
    pub const fn new(value: Value<C>) -> Self {
        Self {
            value,
            attributes: Attributes::new(),
            get: Value::Undefined,
            set: Value::Undefined,
        }
    }

    #[must_use]
    pub const fn getter(value: Value<C>) -> Self {
        Self {
            value: Value::Undefined,
            attributes: Attributes::new(),
            get: value,
            set: Value::Undefined,
        }
    }

    #[must_use]
    pub const fn setter(value: Value<C>) -> Self {
        Self {
            value: Value::Undefined,
            attributes: Attributes::new(),
            get: Value::Undefined,
            set: value,
        }
    }

    pub fn get(self, this: Value<C>, realm: &mut C) -> Result<Value<C>, Error<C>> {
        if self.get.is_nullish() {
            Ok(self.value)
        } else {
            self.get.call(realm, vec![], this)
        }
    }

    pub fn resolve(&self, this: Value<C>, realm: &mut C) -> Result<Value<C>, Error<C>> {
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

    pub fn descriptor(self, obj: &Object<C>) -> Result<(), Error<C>> {
        if self.set.is_undefined() || self.get.is_undefined() {
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

impl<C: Realm> From<Variable<C>> for ObjectProperty<C> {
    fn from(v: Variable<C>) -> Self {
        Self {
            value: v.value,
            attributes: v.properties,
            get: Value::Undefined,
            set: Value::Undefined,
        }
    }
}

// impl<C: Ctx> From<Value<C>> for ObjectProperty<C> {
//     fn from(v: Value<C>) -> Self {
//         Self {
//             value: v,
//             attributes: Attributes::new(),
//             get: Value::Undefined,
//             set: Value::Undefined,
//         }
//     }
// }

impl<C: Realm, V: Into<Value<C>>> From<V> for ObjectProperty<C> {
    fn from(v: V) -> Self {
        Self {
            value: v.into(),
            attributes: Attributes::new(),
            get: Value::Undefined,
            set: Value::Undefined,
        }
    }
}
