mod stack;
mod regs;


pub use stack::*;
pub use regs::*;
use yavashark_bytecode::Instruction;

pub struct VM {
    regs: Registers,
    stack: Stack,
    
    
    pc: usize,
    code: Vec<Instruction>,
}
