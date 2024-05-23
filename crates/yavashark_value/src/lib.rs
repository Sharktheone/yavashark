pub use error::*;
#[cfg(feature = "js")]
pub use js::*;
#[cfg(feature = "ts")]
pub use ts::*;

mod error;

#[cfg(all(feature = "ts", feature = "js"))]
compile_error!("Cannot enable both `ts` and `js` features at the same time");

#[cfg(feature = "ts")]
mod ts;

#[cfg(feature = "js")]
mod js;
