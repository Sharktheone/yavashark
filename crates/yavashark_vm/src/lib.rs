mod execute;
mod instructions;
mod regs;
mod stack;
mod storage;
mod value_ext;

pub use regs::*;
pub use stack::*;
use yavashark_bytecode::Instruction;
use yavashark_env::{Context, Res};
use yavashark_env::scope::Scope;

pub struct VM {
    regs: Registers,
    stack: Stack,

    pc: usize,
    code: Vec<Instruction>,
    
    current_scope: Scope,
    
    ctx: Context,
}

impl VM {
    pub fn get_context(&mut self) -> &mut Context {
        &mut self.ctx
    }

    pub fn push_scope(&mut self) -> Res {
        self.current_scope = self.current_scope.child()?;
        
        Ok(())
    }

    pub fn pop_scope(&mut self) -> Res {
        self.current_scope = self.current_scope.parent()?.ok_or("No parent scope")?;
        
        Ok(())
    }

    pub fn set_pc(&mut self, pc: usize) {
        self.pc = pc;
    }

    pub fn offset_pc(&mut self, offset: isize) {
        // pc won't be above isize::MAX, since this is `Vec`'s length limit
        self.pc = (self.pc as isize + offset) as usize;
    }
}
