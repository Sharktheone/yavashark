pub mod harness;
mod metadata;
pub mod run;
mod test262;
mod utils;
pub mod parsers;

pub type Value = yavashark_value::Value<Realm>;
pub type Error = yavashark_value::Error<Realm>;
pub type ObjectHandle = yavashark_value::Object<Realm>;
pub type Variable = yavashark_value::variable::Variable<Realm>;
pub type Symbol = yavashark_value::Symbol;

pub type ObjectProperty = yavashark_value::ObjectProperty<Realm>;

pub type NativeFunction = yavashark_env::NativeFunction;

pub use yavashark_env::object;
use yavashark_env::Realm;

const TEST262_DIR: &str = "../../test262";
