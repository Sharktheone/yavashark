pub mod async_generator;
mod consts;
mod data;
mod execute;
mod execute_old;
pub mod function_code;
pub mod generator;
mod instruction;
mod instructions;
mod params;
mod regs;
mod stack;
mod storage;
mod task;
mod value_ext;
mod vm;
pub mod bytecode_function;
pub mod async_bytecode_function;

pub use regs::*;
pub use stack::*;

pub use yavashark_bytecode;

use crate::async_generator::AsyncGenerator;
use crate::generator::Generator;
pub use vm::*;
use yavashark_env::{Realm, Res};

pub fn init(realm: &mut Realm) -> Res {
    Generator::init(realm)?;
    AsyncGenerator::init(realm)
}
