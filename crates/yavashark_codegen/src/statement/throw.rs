use crate::{ByteCodegen, Res};
use swc_ecma_ast::{ExprStmt, ThrowStmt};

impl ByteCodegen {
    pub fn compile_throw(&mut self, stmt: &ThrowStmt) -> Res {
        todo!()
    }
}
