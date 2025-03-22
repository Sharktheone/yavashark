//! This crate contains bytecode definitions
//! it does not provide any way to execute or compile it.
//!
//!
//!

pub use consts::*;
pub use instructions_old::*;

mod consts;
pub mod data;
pub mod function;
mod instructions_old;
pub mod writer;
mod instructions;

pub type VarName = u32;
pub type ConstIdx = u32;
pub type Reg = u8;
pub type JmpOffset = isize;
pub type JmpAddr = usize;

pub struct Bytecode {
    pub instructions: Vec<Instruction>,
    pub consts: Vec<ConstValue>,
    pub vars: Vec<String>,
}
