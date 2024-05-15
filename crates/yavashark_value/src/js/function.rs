use std::cell::{Ref, RefCell, RefMut};
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::rc::Rc;

use crate::{Obj, Value};
use crate::error::Error;
use crate::js::context::Ctx;

pub trait AsObject<C: Ctx> {
    fn as_object(&self) -> &dyn Obj<C>;
    fn as_object_mut(&mut self) -> &mut dyn Obj<C>;
}

impl<T: Obj<C>, C: Ctx> AsObject<C> for T {
    fn as_object(&self) -> &dyn Obj<C> {
        self
    }

    fn as_object_mut(&mut self) -> &mut dyn Obj<C> {
        self
    }
}

pub trait Func<C: Ctx>: Debug + Obj<C> + AsObject<C> {
    fn call(
        &mut self,
        ctx: &mut C,
        args: Vec<Value<C>>,
        this: Value<C>,
    ) -> Result<Value<C>, Error<C>>;

    fn into_func_value(self) -> Value<C>
        where
            Self: Sized + 'static,
    {
        let boxed: Box<dyn Func<C>> = Box::new(self);
        Value::Function(Function::from(boxed))
    }
}

#[derive(Debug, Clone)]
pub struct Function<C>(Rc<RefCell<Box<dyn Func<C>>>>);

impl<C: Ctx> Eq for Function<C> {}

impl<C: Ctx> Hash for Function<C> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.0).hash(state); //TODO only the ptr is hashed, not the content
    }
}

impl<C: Ctx> PartialEq for Function<C> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<C: Ctx> Function<C> {
    pub fn get(&self) -> Result<Ref<Box<dyn Func<C>>>, Error<C>> {
        self.0
            .try_borrow()
            .map_err(|_| Error::new("failed to borrow function"))
    }

    pub fn get_mut(&self) -> Result<RefMut<Box<dyn Func<C>>>, Error<C>> {
        self.0
            .try_borrow_mut()
            .map_err(|_| Error::new("failed to borrow function"))
    }

    pub fn call(
        &self,
        ctx: &mut C,
        args: Vec<Value<C>>,
        this: Value<C>,
    ) -> Result<Value<C>, Error<C>> {
        self.get_mut()?.call(ctx, args, this)
    }

    pub fn define_property(&self, name: Value<C>, value: Value<C>) -> Result<(), Error<C>> {
        self.get_mut()?.define_property(name, value);
        Ok(())
    }

    #[must_use]
    pub fn resolve_property(&self, name: &Value<C>) -> Option<Value<C>> {
        self.get().ok()?.resolve_property(name)
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
            .map_or_else(|_| "Function".to_string(), |o| o.name())
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

    pub fn exchange(&self, other: Box<dyn Func<C>>) {
        *self.0.borrow_mut() = other;
    }
}

impl<C: Ctx> Display for Function<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(o) = self.get() {
            write!(f, "{}", o.to_string())
        } else {
            write!(f, "[Function <unknown>]")
        }
    }
}

impl<C: Ctx> From<Box<dyn Func<C>>> for Function<C> {
    fn from(f: Box<dyn Func<C>>) -> Self {
        Self(Rc::new(RefCell::new(f)))
    }
}

impl<C: Ctx> From<Rc<RefCell<Box<dyn Func<C>>>>> for Function<C> {
    fn from(f: Rc<RefCell<Box<dyn Func<C>>>>) -> Self {
        Self(f)
    }
}
