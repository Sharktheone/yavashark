mod node;
mod compiler;
mod data;

pub use compiler::*;

pub type CompileError = anyhow::Error;
pub type Res<T = ()> = Result<T, CompileError>;
