//! This crate contains bytecode definitions
//! it does not provide any way to execute or compile it.
//!
//!
//!

pub use consts::*;
pub use instructions::*;

mod consts;
mod instructions;
pub mod writer;

pub type VarName = u32;
pub type ConstIdx = u32;
pub type Reg = u8;
pub type JmpOffset = i32;

pub struct Bytecode {
    pub instructions: Vec<Instruction>,
    pub consts: Vec<ConstValue>,
    pub vars: Vec<String>,
}
