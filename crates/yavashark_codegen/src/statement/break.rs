use swc_ecma_ast::BreakStmt;
use yavashark_bytecode::Instruction;
use crate::{ByteCodegen, Res};

impl ByteCodegen {
    pub fn compile_break(&mut self, stmt: &BreakStmt) -> Res {
        self.instructions.push(Instruction::Break);
        Ok(())
    }
}
