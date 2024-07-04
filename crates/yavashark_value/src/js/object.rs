use std::any::Any;
use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
#[cfg(feature = "dbg_object_gc")]
use std::sync::atomic::AtomicIsize;

use yavashark_garbage::{Collectable, Gc, GcRef};
use yavashark_garbage::collectable::{CellCollectable, GcMutRefCellGuard, GcRefCellGuard};

use crate::Error;
use crate::js::context::Ctx;
use crate::variable::Variable;

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

    fn resolve_property(&self, name: &Value<C>) -> Option<Value<C>>;

    fn get_property(&self, name: &Value<C>) -> Option<&Value<C>>;

    fn get_property_mut(&mut self, name: &Value<C>) -> Option<&mut Value<C>>;

    fn update_or_define_property(&mut self, name: Value<C>, value: Value<C>) -> Option<Value<C>> {
        if let Some(prop) = self.get_property_mut(&name) {
            let old = prop.clone();
            *prop = value;
            Some(old)
        } else {
            self.define_property(name, value);
            None
        }
    }

    fn define_getter(&mut self, name: Value<C>, value: Value<C>) -> Result<(), Error<C>>;
    fn define_setter(&mut self, name: Value<C>, value: Value<C>) -> Result<(), Error<C>>;
    fn get_getter(&self, name: &Value<C>) -> Option<Value<C>>;
    fn get_setter(&self, name: &Value<C>) -> Option<Value<C>>;

    fn delete_property(&mut self, name: &Value<C>) -> Option<Value<C>>;

    fn contains_key(&self, name: &Value<C>) -> bool {
        self.get_property(name).is_some()
    }

    fn name(&self) -> String;

    fn to_string(&self) -> String;

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

    fn prototype(&self) -> Value<C> {
        self.resolve_property(&"__proto__".into())
            .unwrap_or(Value::Undefined)
    }

    fn constructor(&self) -> Value<C> {
        self.resolve_property(&"constructor".into())
            .unwrap_or(Value::Undefined)
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

        if let Value::Object(o) = self.0.prototype() {
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

    pub fn resolve_property(&self, name: &Value<C>, ctx: &mut C) -> Result<Option<Value<C>>, Error<C>> {
        let this = self.get()?;

        let Some(val) = this.resolve_property(name) else {
            return Ok(if let Some(getter) = this.get_getter(name) {
                drop(this);
                let this = Value::Object(self.clone());
                
                Some(getter.call(ctx, Vec::new(), this)?)
            } else {
                None
            });
        };

        Ok(Some(val))
    }
    pub fn resolve_property_no_get_set(&self, name: &Value<C>) -> Result<Option<Value<C>>, Error<C>> {
        let this = self.get()?;
        
        Ok(this.resolve_property(name))
    }


        pub fn get_mut(&self) -> Result<GcMutRefCellGuard<BoxedObj<C>>, Error<C>> {
        self.0
            .borrow_mut()
            .map_err(|_| Error::new("failed to borrow object"))
    }

    pub fn define_property(&self, name: Value<C>, value: Value<C>) -> Result<(), Error<C>> {
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

    pub fn update_or_define_property(
        &self,
        name: &Value<C>,
        value: Value<C>,
    ) -> Result<(), Error<C>> {
        let mut inner = self.get_mut()?;

        inner.update_or_define_property(name.copy(), value);

        Ok(())
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
    pub fn get_constructor(&self) -> Value<C> {
        self.get().map_or(Value::Undefined, |o| o.constructor())
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
}

impl<C: Ctx> Display for Object<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Ok(o) = self.get() {
            write!(f, "{}", o.to_string())
        } else {
            write!(f, "[object Object]")
        }
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
}
