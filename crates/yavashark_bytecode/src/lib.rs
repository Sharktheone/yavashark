//! This crate contains bytecode definitions
//! it does not provide any way to execute or compile it.
//!
//!
//!

pub use instructions::*;
use yavashark_env::Value;

pub mod writer;
mod instructions;

pub type VarName = u32;
pub type ConstIdx = u32;
pub type Reg = u8;
pub type JmpOffset = u32;

pub struct Bytecode {
    pub instructions: Vec<Instruction>,
    pub consts: Vec<Value>,
    pub vars: Vec<String>,
}