use std::any::Any;
use std::cell::{Ref, RefCell, RefMut};
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::rc::Rc;

use crate::js::context::Ctx;
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

    fn resolve_property(&self, name: &Value<C>) -> Option<Value<C>>;

    fn get_property(&self, name: &Value<C>) -> Option<&Value<C>>;
    //
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
}

#[derive(Debug, Clone)]
pub struct Object<C: Ctx>(Rc<RefCell<Box<dyn Obj<C>>>>);

impl<C: Ctx> Hash for Object<C> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.0).hash(state); //TODO only the ptr is hashed, not the content
    }
}

impl<C: Ctx> Eq for Object<C> {}

impl<C: Ctx> PartialEq for Object<C> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<C: Ctx> Object<C> {
    pub fn get(&self) -> Result<Ref<Box<dyn Obj<C>>>, Error<C>> {
        self.0
            .try_borrow()
            .map_err(|_| Error::new("failed to borrow object"))
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

    pub fn get_property(&self, name: &Value<C>) -> Result<Value<C>, Error<C>> {
        self.get()?
            .get_property(name)
            .map(|v| v.copy())
            .ok_or(Error::reference_error(format!(
                "{} does not exist on object",
                name
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

    pub fn name(&self) -> String {
        if let Ok(o) = self.get() {
            o.name().to_string()
        } else {
            "Object".to_string()
        }
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
        Object(Rc::new(RefCell::new(obj)))
    }
}

impl<C: Ctx> Object<C> {
    pub fn new(obj: Box<dyn Obj<C>>) -> Self {
        Object(Rc::new(RefCell::new(obj)))
    }
}
