use std::any::Any;
use std::cell::RefCell;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
#[cfg(feature = "dbg_object_gc")]
use std::sync::atomic::AtomicIsize;

use yavashark_garbage::collectable::{CellCollectable, GcMutRefCellGuard, GcRefCellGuard};
use yavashark_garbage::{Collectable, Gc, GcRef};

use crate::js::context::Ctx;
use crate::variable::Variable;
use crate::{Attributes, Error};

use super::Value;

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

pub trait Obj<C: Ctx>: Debug + AsAny {
    fn define_property(&mut self, name: Value<C>, value: Value<C>);

    fn define_variable(&mut self, name: Value<C>, value: Variable<C>);

    fn resolve_property(&self, name: &Value<C>) -> Option<ObjectProperty<C>>;

    fn get_property(&self, name: &Value<C>) -> Option<&Value<C>>;

    fn define_getter(&mut self, name: Value<C>, value: Value<C>) -> Result<(), Error<C>>;
    fn define_setter(&mut self, name: Value<C>, value: Value<C>) -> Result<(), Error<C>>;
    fn get_getter(&self, name: &Value<C>) -> Option<Value<C>>;
    fn get_setter(&self, name: &Value<C>) -> Option<Value<C>>;

    fn delete_property(&mut self, name: &Value<C>) -> Option<Value<C>>;

    fn contains_key(&self, name: &Value<C>) -> bool {
        self.get_property(name).is_some()
    }

    fn name(&self) -> String;

    fn to_string(&self, ctx: &mut C) -> Result<String, Error<C>>;
    fn to_string_internal(&self) -> String;

    fn properties(&self) -> Vec<(Value<C>, Value<C>)>;

    fn keys(&self) -> Vec<Value<C>>;

    fn values(&self) -> Vec<Value<C>>;

    fn into_object(self) -> Object<C>
    where
        Self: Sized + 'static,
    {
        let boxed: Box<dyn Obj<C>> = Box::new(self);

        Object::from_boxed(boxed)
    }

    fn into_value(self) -> Value<C>
    where
        Self: Sized + 'static,
    {
        Value::Object(self.into_object())
    }

    fn get_array_or_done(&self, index: usize) -> (bool, Option<Value<C>>);

    fn clear_values(&mut self);

    #[allow(unused_variables)]
    fn call(
        &mut self,
        ctx: &mut C,
        args: Vec<Value<C>>,
        this: Value<C>,
    ) -> Result<Value<C>, Error<C>> {
        Err(Error::ty_error(format!(
            "{} is not a function",
            self.name()
        )))
    }

    fn is_function(&self) -> bool {
        false
    }

    fn prototype(&self) -> ObjectProperty<C> {
        self.resolve_property(&"__proto__".into())
            .unwrap_or(Value::Undefined.into())
    }

    fn constructor(&self) -> ObjectProperty<C> {
        self.resolve_property(&"constructor".into())
            .unwrap_or(Value::Undefined.into())
    }

    /// # Safety
    /// This function should only return references that are actually in the object!
    /// Else it will leak memory and cause undefined behavior, same for references that are in the object but not known to the gc!
    unsafe fn custom_gc_refs(&self) -> Vec<GcRef<RefCell<BoxedObj<C>>>> {
        Vec::new()
    }

    fn class_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn get_constructor_value(&self, _ctx: &mut C) -> Option<Value<C>> {
        None
    }

    fn get_constructor_proto(&self, _ctx: &mut C) -> Option<Value<C>> {
        None
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
pub struct BoxedObj<C: Ctx>(Box<dyn Obj<C>>);

impl<C: Ctx> Deref for BoxedObj<C> {
    type Target = dyn Obj<C>;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<C: Ctx> DerefMut for BoxedObj<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

unsafe impl<C: Ctx> CellCollectable<RefCell<Self>> for BoxedObj<C> {
    fn get_refs(&self) -> Vec<GcRef<RefCell<Self>>> {
        let properties = self.0.properties();

        let mut refs = Vec::with_capacity(properties.len()); //Not all props will be objects, so we speculate that not all names and values are objects

        self.0.properties().into_iter().for_each(|(n, v)| {
            if let Value::Object(o) = n {
                refs.push(o.0.get_ref());
            }

            if let Value::Object(o) = v {
                refs.push(o.0.get_ref());
            }
        });

        let p = self.0.prototype();

        if let Value::Object(o) = p.value {
            refs.push(o.0.get_ref());
        }

        if let Value::Object(o) = p.get {
            refs.push(o.0.get_ref());
        }

        if let Value::Object(o) = p.set {
            refs.push(o.0.get_ref());
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

impl<C: Ctx> BoxedObj<C> {
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
pub struct Object<C: Ctx>(Gc<RefCell<BoxedObj<C>>>);

#[cfg(any(test, debug_assertions, feature = "display_object"))]
impl<C: Ctx> Display for Object<C> {
    /// This function shouldn't be used in production code, only for debugging
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.get() {
            Ok(s) => write!(f, "{}", s.to_string_internal()),
            Err(e) => write!(f, "Error displaying object: {e}"),
        }
    }
}

#[cfg(feature = "dbg_object_gc")]
impl<C: Ctx> Drop for BoxedObj<C> {
    fn drop(&mut self) {
        OBJECT_COUNT.decrement();
    }
}

impl<C: Ctx> Debug for Object<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", *self)
    }
}

impl<C: Ctx> Hash for Object<C> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_ptr().hash(state);
    }
}

impl<C: Ctx> Eq for Object<C> {}

impl<C: Ctx> PartialEq for Object<C> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<C: Ctx> Object<C> {
    pub fn get(&self) -> Result<GcRefCellGuard<BoxedObj<C>>, Error<C>> {
        self.0
            .borrow()
            .map_err(|_| Error::new("failed to borrow object"))
    }

    pub fn resolve_property(
        &self,
        name: &Value<C>,
        ctx: &mut C,
    ) -> Result<Option<Value<C>>, Error<C>> {
        let this = self.get()?;

        let Some(p) = this.resolve_property(name) else {
            return Ok(None);
        };

        p.get(Value::Object(self.clone()), ctx).map(Some)
    }
    pub fn resolve_property_no_get_set(
        &self,
        name: &Value<C>,
    ) -> Result<Option<ObjectProperty<C>>, Error<C>> {
        let this = self.get()?;

        Ok(this.resolve_property(name))
    }

    pub fn get_mut(&self) -> Result<GcMutRefCellGuard<BoxedObj<C>>, Error<C>> {
        self.0
            .borrow_mut()
            .map_err(|_| Error::new("failed to borrow object"))
    }

    pub fn define_property(&self, name: Value<C>, value: Value<C>) -> Result<(), Error<C>> {
        //TODO: maybe this should be called set_property or something
        // # Safety:
        // We attach the values below
        let mut inner = self.get_mut()?;

        inner.define_property(name, value);

        Ok(())
    }

    pub fn define_variable(&self, name: Value<C>, value: Variable<C>) -> Result<(), Error<C>> {
        let mut inner = self.get_mut()?;

        inner.define_variable(name, value);

        Ok(())
    }

    pub fn get_property(&self, name: &Value<C>) -> Result<Value<C>, Error<C>> {
        self.get()?
            .get_property(name)
            .map(super::Value::copy)
            .ok_or(Error::reference_error(format!(
                "{name} does not exist on object"
            )))
    }

    pub fn contains_key(&self, name: &Value<C>) -> Result<bool, Error<C>> {
        Ok(self.get()?.contains_key(name))
    }

    #[must_use]
    pub fn name(&self) -> String {
        self.get()
            .map_or_else(|_| "Object".to_string(), |o| o.name())
    }

    #[allow(clippy::type_complexity)]
    pub fn properties(&self) -> Result<Vec<(Value<C>, Value<C>)>, Error<C>> {
        Ok(self.get()?.properties())
    }

    pub fn keys(&self) -> Result<Vec<Value<C>>, Error<C>> {
        Ok(self.get()?.keys())
    }

    pub fn values(&self) -> Result<Vec<Value<C>>, Error<C>> {
        Ok(self.get()?.values())
    }

    pub fn exchange(&self, other: Box<dyn Obj<C>>) -> Result<(), Error<C>> {
        **self
            .0
            .borrow_mut()
            .map_err(|_| Error::new("Failed to borrow object"))? = BoxedObj::new(other);

        Ok(())
    }

    pub fn call(
        &self,
        ctx: &mut C,
        args: Vec<Value<C>>,
        this: Value<C>,
    ) -> Result<Value<C>, Error<C>> {
        // # Safety:
        // Since we are not changing the object, we can safely get a mutable reference
        let mut inner = self.get_mut()?;

        inner.call(ctx, args, this)
    }

    #[must_use]
    pub fn is_function(&self) -> bool {
        self.get().map_or(false, |o| o.is_function())
    }

    pub fn clear_values(&self) -> Result<(), Error<C>> {
        let mut inner = self.get_mut()?;

        inner.clear_values();
        Ok(())
    }

    #[must_use]
    pub fn gc_get_ref(&self) -> GcRef<RefCell<BoxedObj<C>>> {
        self.0.get_ref()
    }

    #[must_use]
    pub fn gc_get_untyped_ref<U: Collectable>(&self) -> GcRef<U> {
        self.0.get_untyped_ref()
    }

    #[must_use]
    pub fn custom_refs(&self) -> Vec<GcRef<RefCell<BoxedObj<C>>>> {
        self.get()
            .map_or_else(|_| Vec::new(), |o| unsafe { o.custom_gc_refs() })
    }

    pub fn get_constructor_value(&self, ctx: &mut C) -> Option<Value<C>> {
        self.get().map_or(None, |o| o.get_constructor_value(ctx))
    }

    #[must_use]
    pub fn get_constructor(&self) -> ObjectProperty<C> {
        self.get()
            .map_or(Value::Undefined.into(), |o| o.constructor())
    }

    /// I hate JavaScript...
    pub fn special_constructor(&self) -> Result<bool, Error<C>> {
        self.get()
            .map_or(Err(Error::new("failed to get object")), |o| {
                Ok(o.special_constructor())
            })
    }

    pub fn define_setter(&self, name: Value<C>, value: Value<C>) -> Result<(), Error<C>> {
        let mut inner = self.get_mut()?;

        inner.define_setter(name, value)
    }

    pub fn define_getter(&self, name: Value<C>, value: Value<C>) -> Result<(), Error<C>> {
        let mut inner = self.get_mut()?;

        inner.define_getter(name, value)
    }


    pub fn id(&self) -> usize {
        self.0.ptr_id()
    }
}

impl<C: Ctx> From<Box<dyn Obj<C>>> for Object<C> {
    fn from(obj: Box<dyn Obj<C>>) -> Self {
        Self(Gc::new(RefCell::new(BoxedObj::new(obj))))
    }
}

impl<C: Ctx> Object<C> {
    #[must_use]
    pub fn from_boxed(obj: Box<dyn Obj<C>>) -> Self {
        Self(Gc::new(RefCell::new(BoxedObj::new(obj))))
    }

    pub fn new<O: Obj<C> + 'static>(obj: O) -> Self {
        Self(Gc::new(RefCell::new(BoxedObj::new(Box::new(obj)))))
    }

    pub fn to_string(&self, ctx: &mut C) -> Result<String, Error<C>> {
        self.get()?.to_string(ctx)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ObjectProperty<C: Ctx> {
    pub value: Value<C>,
    pub attributes: Attributes,
    pub get: Value<C>,
    pub set: Value<C>,
}

impl<C: Ctx> ObjectProperty<C> {
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

    pub fn get(self, this: Value<C>, ctx: &mut C) -> Result<Value<C>, Error<C>> {
        if self.get.is_nullish() {
            Ok(self.value)
        } else {
            self.get.call(ctx, vec![], this)
        }
    }

    pub fn resolve(&self, this: Value<C>, ctx: &mut C) -> Result<Value<C>, Error<C>> {
        if self.get.is_nullish() {
            Ok(self.value.copy())
        } else {
            self.get.call(ctx, vec![], this)
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
}

impl<C: Ctx> From<Variable<C>> for ObjectProperty<C> {
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

impl<C: Ctx, V: Into<Value<C>>> From<V> for ObjectProperty<C> {
    fn from(v: V) -> Self {
        Self {
            value: v.into(),
            attributes: Attributes::new(),
            get: Value::Undefined,
            set: Value::Undefined,
        }
    }
}
