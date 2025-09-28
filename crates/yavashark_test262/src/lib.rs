pub mod harness;
mod metadata;
pub mod parsers;
pub mod run;
mod test262;
mod utils;

pub type Value = yavashark_value::Value<Realm>;
pub type Error = yavashark_value::Error<Realm>;
pub type ObjectHandle = yavashark_value::Object<Realm>;
pub type Variable = yavashark_value::variable::Variable<Realm>;
pub type Symbol = yavashark_value::Symbol;

pub type ObjectProperty = yavashark_value::ObjectProperty<Realm>;

pub type NativeFunction = yavashark_env::NativeFunction;

pub use yavashark_env::object;
use yavashark_env::Realm;

const TEST262_FALLBACK_DIR: &str = "../../test262";

#[cfg(feature = "timings")]
pub static mut PARSE_DURATION: std::time::Duration = std::time::Duration::ZERO;
#[cfg(feature = "timings")]
pub static mut SETUP_DURATION: std::time::Duration = std::time::Duration::ZERO;
#[cfg(feature = "timings")]
pub static mut REALM_DURATION: std::time::Duration = std::time::Duration::ZERO;
