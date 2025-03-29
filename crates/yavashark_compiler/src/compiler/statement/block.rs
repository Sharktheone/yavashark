use yavashark_bytecode::instructions::Instruction;
use crate::{Compiler, Res};

impl Compiler {
    pub fn compile_block(&mut self, block: &swc_ecma_ast::BlockStmt) -> Res {
        self.instructions.push(Instruction::PushScope);
        self.compile_stmts(&block.stmts)?;
        self.instructions.push(Instruction::PopScope);
        
        Ok(())
    }
}