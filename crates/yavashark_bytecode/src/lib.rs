//! This crate contains bytecode definitions
//! it does not provide any way to execute or compile it.
//!
//!
//!

pub use consts::*;
pub use instructions_old::*;

mod constructor;
mod consts;
pub mod data;
pub mod function;
pub mod instructions;
mod instructions_old;
pub mod jmp;
pub mod writer;

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
