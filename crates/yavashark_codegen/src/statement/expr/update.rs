use crate::{ByteCodegen, Res};
use swc_ecma_ast::{ThisExpr, UpdateExpr};

impl ByteCodegen {
    pub fn compile_update(&mut self, stmt: &UpdateExpr) -> Res {
        todo!()
    }
}
