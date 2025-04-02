mod compiler;
mod data;
mod node;

pub use compiler::*;

pub type CompileError = anyhow::Error;
pub type Res<T = ()> = Result<T, CompileError>;
