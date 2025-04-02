use crate::{Compiler, Res};
use yavashark_bytecode::instructions::Instruction;

impl Compiler {
    pub fn compile_block(&mut self, block: &swc_ecma_ast::BlockStmt) -> Res {
        self.instructions.push(Instruction::PushScope);
        self.compile_stmts(&block.stmts)?;
        self.instructions.push(Instruction::PopScope);

        Ok(())
    }
}
