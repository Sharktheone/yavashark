pub use console::*;
pub use context::*;
pub use function::*;
pub use object::*;

mod console;
mod context;
pub mod error;
mod function;
pub mod object;
pub mod scope;

pub mod realm;
#[cfg(feature = "tests")]
pub mod tests;

use crate::error::ErrorObj;
pub use yavashark_value as value;

pub type Value = yavashark_value::Value<Context>;
pub type Error = yavashark_value::Error<Context>;
pub type ObjectHandle = yavashark_value::Object<Context>;
pub type Variable = yavashark_value::variable::Variable<Context>;
pub type Symbol = yavashark_value::Symbol<Context>;

pub type ObjectProperty = yavashark_value::ObjectProperty<Context>;

#[derive(Debug, PartialEq, Eq)]
pub enum ControlFlow {
    Continue(Option<String>),
    Break(Option<String>),
    Return(Value),
    Error(Error),
    OptChainShortCircuit,
}

impl ControlFlow {
    #[must_use]
    pub const fn error(e: String) -> Self {
        Self::Error(Error::new_error(e))
    }

    #[must_use]
    pub const fn error_reference(e: String) -> Self {
        Self::Error(Error::reference_error(e))
    }
    #[must_use]
    pub fn error_syntax(e: &str) -> Self {
        Self::Error(Error::syn(e))
    }
    #[must_use]
    pub const fn error_type(e: String) -> Self {
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
                if let Ok(obj) = obj.get() {
                    let this = (***obj).as_any();

                    if let Some(err) = this.downcast_ref::<ErrorObj>() {
                        let e = &err.error;

                        return Self::Error(e.clone());
                    }
                }
            }
        }

        Self::Error(Error::throw(val))
    }
}

pub type ValueResult = std::result::Result<Value, Error>;

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub type Res = Result<()>;

pub type RuntimeResult = std::result::Result<Value, ControlFlow>;

pub type ControlResult = std::result::Result<(), ControlFlow>;

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
