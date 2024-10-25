use yavashark_env::Context;

mod test262;

pub type Value = yavashark_value::Value<Context>;
pub type Error = yavashark_value::Error<Context>;
pub type ObjectHandle = yavashark_value::Object<Context>;
pub type Variable = yavashark_value::variable::Variable<Context>;
pub type Symbol = yavashark_value::Symbol<Context>;

pub type ObjectProperty = yavashark_value::ObjectProperty<Context>;

pub type NativeFunction = yavashark_env::NativeFunction;

pub use yavashark_env::object;
