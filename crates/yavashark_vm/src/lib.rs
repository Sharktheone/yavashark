mod execute;
mod instructions;
mod regs;
mod stack;
mod storage;

pub use regs::*;
pub use stack::*;
use yavashark_bytecode::Instruction;
use yavashark_env::Context;

pub struct VM {
    regs: Registers,
    stack: Stack,

    pc: usize,
    code: Vec<Instruction>,
}

impl VM {
    pub fn get_context(&mut self) -> &mut Context {
        todo!()
    }
    
    
    pub fn push_scope(&mut self) {
        todo!()
    }
    
    pub fn pop_scope(&mut self) {
        todo!()
    }
}
