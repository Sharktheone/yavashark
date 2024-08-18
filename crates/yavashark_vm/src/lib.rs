mod execute;
mod regs;
mod stack;
mod instructions;
mod storage;

pub use regs::*;
pub use stack::*;
use yavashark_bytecode::Instruction;

pub struct VM {
    regs: Registers,
    stack: Stack,

    pc: usize,
    code: Vec<Instruction>,
}
