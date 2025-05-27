#[cfg(target_arch = "wasm32")]
mod wasm;
mod direct_exec;

pub use direct_exec::*;
