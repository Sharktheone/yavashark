use crate::{ByteCodegen, Res};
use swc_ecma_ast::BreakStmt;
use yavashark_bytecode::Instruction;

impl ByteCodegen {
    pub fn compile_break(&mut self, stmt: &BreakStmt) -> Res {
        self.instructions.push(Instruction::Break);
        Ok(())
    }
}
