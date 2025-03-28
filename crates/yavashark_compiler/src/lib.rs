mod node;
mod compiler;



pub use compiler::*;

pub type CompileError = anyhow::Error;
pub type Res = Result<(), CompileError>;
