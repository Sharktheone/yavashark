#![allow(clippy::future_not_send)]

pub use console::*;
pub use function::*;
pub use native_object::*;
pub use object::*;
use std::ops::Deref;
use yavashark_garbage::OwningGcGuard;

pub mod console;
pub mod error;
mod function;
pub mod object;
pub mod scope;

pub mod args;
pub mod builtins;
pub mod conversion;
#[cfg(feature = "out-of-spec-experiments")]
pub mod experiments;
mod function_conversion;
mod global;
mod native_obj;
mod native_object;
pub mod optimizer;
pub mod realm;
pub mod task_queue;
#[cfg(feature = "tests")]
pub mod tests;
pub mod utils;

use crate::error::ErrorObj;
pub use crate::realm::Realm;
pub use yavashark_value as value;
use yavashark_value::{BoxedObj, Location};

pub type Value = yavashark_value::Value<Realm>;
pub type Error = yavashark_value::Error<Realm>;
pub type ObjectHandle = yavashark_value::Object<Realm>;
pub type Variable = yavashark_value::variable::Variable<Realm>;
pub type Symbol = yavashark_value::Symbol;

pub type ObjectProperty = yavashark_value::ObjectProperty<Realm>;

#[derive(Debug, PartialEq, Eq)]
pub enum ControlFlow {
    Continue(Option<String>),
    Break(Option<String>),
    Return(Value),
    Error(Error),
    Yield(Value),
    Await(ObjectHandle),
    OptChainShortCircuit,
}

impl ControlFlow {
    #[must_use]
    pub fn error(e: String) -> Self {
        Self::Error(Error::new_error(e))
    }

    #[must_use]
    pub fn error_reference(e: String) -> Self {
        Self::Error(Error::reference_error(e))
    }
    #[must_use]
    pub const fn error_syn(e: &'static str) -> Self {
        Self::Error(Error::syn(e))
    }

    #[must_use]
    pub fn error_syntax(e: String) -> Self {
        Self::Error(Error::syn_error(e))
    }

    #[must_use]
    pub fn error_type(e: String) -> Self {
        Self::Error(Error::ty_error(e))
    }

    pub fn get_error(self) -> std::result::Result<Error, Self> {
        match self {
            Self::Error(e) => Ok(e),
            e => Err(e),
        }
    }

    #[must_use]
    pub fn throw(val: Value) -> Self {
        if let Value::Object(obj) = &val {
            {
                let obj = obj.guard();
                let this = (**obj).as_any();

                if let Some(err) = this.downcast_ref::<ErrorObj>() {
                    let inner = match err.inner.try_borrow() {
                        Ok(inner) => inner,
                        Err(e) => {
                            return Self::Error(e.into());
                        }
                    };

                    let e = &inner.error;

                    return Self::Error(e.clone());
                }
            }
        }

        Self::Error(Error::throw(val))
    }

    pub fn attach_location(&mut self, loc: Location) {
        if let Self::Error(e) = self {
            e.attach_location(loc);
        }
    }
}

pub type ValueResult = Result<Value, Error>;

pub type Res<T = (), E = Error> = Result<T, E>;

pub type RuntimeResult = Result<Value, ControlFlow>;

pub type ControlResult = Result<(), ControlFlow>;

pub enum RefOrOwned<T: 'static> {
    Ref(OwningGcGuard<'static, BoxedObj<Realm>, T>),
    Owned(T),
}

impl<T> Deref for RefOrOwned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Ref(r) => r,
            Self::Owned(o) => o,
        }
    }
}

impl From<Error> for ControlFlow {
    fn from(e: Error) -> Self {
        Self::Error(e)
    }
}

impl From<ControlFlow> for Error {
    fn from(e: ControlFlow) -> Self {
        match e {
            ControlFlow::Error(e) => e,
            _ => Self::new("Incorrect ControlFlow"),
        }
    }
}
