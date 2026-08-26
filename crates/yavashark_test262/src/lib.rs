pub mod harness;
mod metadata;
pub mod parsers;
pub mod run;
mod test262;
mod utils;

pub use yavashark_env::value::Value;
pub use yavashark_env::error::Error;
pub use yavashark_env::value::Object as ObjectHandle;
pub use yavashark_env::value::variable::Variable;
pub use yavashark_env::value::Symbol;

pub use yavashark_env::value::ObjectProperty;

pub use yavashark_env::NativeFunction;

pub use yavashark_env::object;

const TEST262_FALLBACK_DIR: &str = "../../test262";

#[cfg(feature = "timings")]
pub static mut PARSE_DURATION: std::time::Duration = std::time::Duration::ZERO;
#[cfg(feature = "timings")]
pub static mut SETUP_DURATION: std::time::Duration = std::time::Duration::ZERO;
#[cfg(feature = "timings")]
pub static mut REALM_DURATION: std::time::Duration = std::time::Duration::ZERO;
