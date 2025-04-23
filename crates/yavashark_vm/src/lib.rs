mod consts;
mod data;
mod execute;
mod execute_old;
pub mod function_code;
mod instruction;
mod instructions;
mod regs;
mod stack;
mod storage;
mod task;
mod value_ext;
mod vm;
pub mod generator;
pub mod async_generator;

pub use regs::*;
pub use stack::*;

pub use yavashark_bytecode;

pub use vm::*;
use yavashark_env::{Realm, Res};
use crate::generator::Generator;

pub fn init(realm: &mut Realm) -> Res {
    Generator::init(realm)
}