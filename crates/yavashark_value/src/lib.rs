#[cfg(feature = "ts")]
pub use ts::*;

#[cfg(all(feature = "ts", feature = "js"))]
compile_error!("Cannot enable both `ts` and `js` features at the same time");

#[cfg(feature = "ts")]
mod ts;

