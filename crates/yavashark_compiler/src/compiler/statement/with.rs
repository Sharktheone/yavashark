use swc_ecma_ast::WithStmt;
use yavashark_bytecode::instructions::Instruction;
use crate::{Compiler, Res};

impl Compiler {
    pub fn compile_with(&mut self, w: &WithStmt) -> Res {
        let obj = self.compile_expr_data_acc(&w.obj)?;
        
        self.instructions.push(Instruction::PushScope);
        self.instructions.push(Instruction::with(obj));
        self.instructions.push(Instruction::PopScope);
        
        Ok(())
    }
    
}