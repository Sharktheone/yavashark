mod execute;
mod instructions;
mod regs;
mod stack;
mod storage;
mod value_ext;
pub mod function_code;
mod vm;

use crate::execute::Execute;
pub use regs::*;
pub use stack::*;
use std::path::PathBuf;
use yavashark_bytecode::data::DataSection;
use yavashark_bytecode::{Instruction, Reg, VarName};
use yavashark_env::scope::{ParentOrGlobal, Scope};
use yavashark_env::Result;
use yavashark_env::{Error, Realm, Res, Value};

pub use yavashark_bytecode;

pub use vm::*;
