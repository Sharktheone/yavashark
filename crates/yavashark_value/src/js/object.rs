use std::any::Any;
use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
#[cfg(feature = "dbg_object_gc")]
use std::sync::atomic::AtomicIsize;

use yavashark_garbage::{Gc, GcRef};
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

    fn prototype(&self) -> &Value<C> {
        self.get_property(&"__proto__".into()).unwrap_or(&Value::Undefined)
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

        refs
    }

    fn get_refs_diff(&self, old: &[GcRef<RefCell<Self>>]) -> (Vec<GcRef<RefCell<Self>>>, Vec<GcRef<RefCell<Self>>>) {
        let mut properties = self.0.properties();

        let mut refs = Vec::with_capacity(properties.len()); //Not all props will be objects, so we speculate that not all names and values are objects 


        properties.iter().for_each(|(n, v)| {
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
       

        let remove = old.iter().filter(|x| !refs.contains(x)).cloned().collect();
        let add = refs.into_iter().filter(|x| !old.contains(x)).collect();

        (add, remove)
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
        self.0.borrow()
            .map_err(|_| Error::new("failed to borrow object"))
    }

    #[must_use]
    pub fn resolve_property(&self, name: &Value<C>) -> Option<Value<C>> {
        self.get().ok()?.resolve_property(name)
    }

    pub fn get_mut(&self) -> Result<GcMutRefCellGuard<BoxedObj<C>>, Error<C>> {
        self.0.borrow_mut()
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
        **self.0.borrow_mut().map_err(|_| Error::new("Failed to borrow object"))? = BoxedObj::new(other);

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
