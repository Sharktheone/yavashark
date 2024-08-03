use crate::{ByteCodegen, Res};
use swc_ecma_ast::{ContinueStmt, ExprStmt};
use yavashark_bytecode::Instruction;

impl ByteCodegen {
    pub fn compile_continue(&mut self, stmt: &ContinueStmt) -> Res {
        self.instructions.push(Instruction::Continue);
        Ok(())
    }
}
