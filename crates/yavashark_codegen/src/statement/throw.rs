use crate::{ByteCodegen, Res};
use swc_ecma_ast::{ExprStmt, ThrowStmt};
use yavashark_bytecode::Instruction;

impl ByteCodegen {
    pub fn compile_throw(&mut self, stmt: &ThrowStmt) -> Res {
        self.compile_expr(&stmt.arg, stmt.span);

        self.instructions.push(Instruction::ThrowAcc);

        Ok(())
    }
}
