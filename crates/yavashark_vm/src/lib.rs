mod execute;
mod instructions;
mod regs;
mod stack;
mod storage;
mod value_ext;

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

    pub fn set_pc(&mut self, pc: usize) {
        self.pc = pc;
    }

    pub fn offset_pc(&mut self, offset: isize) {
        // pc won't be above isize::MAX, since this is `Vec`'s length limit
        self.pc = (self.pc as isize + offset) as usize;
    }
}
