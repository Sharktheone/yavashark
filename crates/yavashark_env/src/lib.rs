#![allow(clippy::future_not_send)]

pub use console::*;
pub use function::*;
pub use native_object::*;
pub use object::*;
use std::ops::Deref;
use yavashark_garbage::OwningGcGuard;

pub mod console;
pub mod error_obj;
mod function;
pub mod object;
pub mod scope;

pub mod args;
pub mod builtins;
pub mod conversion;
pub mod error;
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
// #[cfg(feature = "js")]
pub mod import;
pub mod inline_props;
pub mod value;

pub mod partial_init;
mod iter;

use crate::error_obj::ErrorObj;
pub use crate::realm::Realm;
pub use crate::value::property_key::{InternalPropertyKey, PropertyKey};
pub use crate::value::{ObjectOrNull, PrimitiveValue};
use error::Location;
use value::BoxedObj;

pub type Value = value::Value;
pub type WeakValue = value::WeakValue;
pub type Error = error::Error;
pub type ObjectHandle = value::Object;
pub type WeakObjectHandle = value::WeakObject;
pub type Variable = value::variable::Variable;
pub type Symbol = value::Symbol;

pub type ObjectProperty = value::ObjectProperty;

pub type GCd<T> = OwningGcGuard<'static, BoxedObj, T>;

type PreHashedPropertyKey = (InternalPropertyKey, u64);

#[derive(Debug, PartialEq, Eq)]
pub enum ControlFlow {
    Continue(Option<String>),
    Break(Option<String>),
    Return(Value),
    Error(Error),
    Yield(Value),
    YieldStar(ObjectHandle),
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
                if let Some(err) = obj.downcast::<ErrorObj>() {
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
    Ref(OwningGcGuard<'static, BoxedObj, T>),
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
