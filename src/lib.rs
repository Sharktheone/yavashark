mod direct_exec;
#[cfg(target_arch = "wasm32")]
mod wasm;

pub use direct_exec::*;
