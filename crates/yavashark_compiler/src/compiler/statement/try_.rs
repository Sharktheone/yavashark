use crate::{Compiler, Res};
use swc_ecma_ast::TryStmt;
use yavashark_bytecode::instructions::Instruction;

impl Compiler {
    pub fn compile_try(&mut self, s: &TryStmt) -> Res {
        
        self.instructions.push(Instruction::enter_try())
        
        todo!()
    }
}
