use std::any::Any;
use std::cell::{Ref, RefCell, RefMut};
use std::fmt::{Debug, Display};
use std::hash::Hash;
use yavashark_garbage::Gc;

use crate::js::context::Ctx;
use crate::variable::Variable;
use crate::Error;

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

    fn update_or_define_property(&mut self, name: Value<C>, value: Value<C>) {
        if let Some(prop) = self.get_property_mut(&name) {
            *prop = value;
        } else {
            self.define_property(name, value);
        }
    }

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

        Object::new(boxed)
    }

    fn into_value(self) -> Value<C>
    where
        Self: Sized + 'static,
    {
        Value::Object(self.into_object())
    }

    fn get_array_or_done(&self, index: usize) -> (bool, Option<Value<C>>);
}

#[derive(Clone)]
pub struct Object<C: Ctx>(pub Gc<RefCell<Box<dyn Obj<C>>>>);

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
        self.0.as_ptr() == other.0.as_ptr()
    }
}

impl<C: Ctx> Object<C> {
    pub fn get(&self) -> Result<Ref<Box<dyn Obj<C>>>, Error<C>> {
        self.0
            .try_borrow()
            .map_err(|_| Error::new("failed to borrow object"))
    }

    #[must_use]
    pub fn resolve_property(&self, name: &Value<C>) -> Option<Value<C>> {
        self.get().ok()?.resolve_property(name)
    }

    pub fn get_mut(&self) -> Result<RefMut<Box<dyn Obj<C>>>, Error<C>> {
        self.0
            .try_borrow_mut()
            .map_err(|_| Error::new("failed to borrow object"))
    }

    pub fn define_property(&self, name: Value<C>, value: Value<C>) -> Result<(), Error<C>> {
        self.get_mut()?.define_property(name, value);
        Ok(())
    }

    pub fn define_variable(&self, name: Value<C>, value: Variable<C>) -> Result<(), Error<C>> {
        self.get_mut()?.define_variable(name, value);
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
        name: Value<C>,
        value: Value<C>,
    ) -> Result<(), Error<C>> {
        self.get_mut()?.update_or_define_property(name, value);
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

    pub fn exchange(&self, other: Box<dyn Obj<C>>) {
        *self.0.borrow_mut() = other;
    }
    pub fn call(&self, ctx: &mut C, args: Vec<Value<C>>, this: Value<C>) -> Result<Value<C>, Error<C>> {
        // # Safety:
        // Since we are not changing the object, we can safely get a mutable reference
        let mut inner = unsafe { self.get_mut()? };

        inner.call(ctx, args, this)
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
        Self(Gc::new(RefCell::new(obj)))
    }
}

impl<C: Ctx> Object<C> {
    #[must_use]
    pub fn from_boxed(obj: Box<dyn Obj<C>>) -> Self {
        Self(Gc::new(RefCell::new(obj)))
    }


    pub fn new<O: Obj<C> + 'static>(obj: O) -> Self {
        Self(Gc::new(RefCell::new(Box::new(obj))))
    }
}
