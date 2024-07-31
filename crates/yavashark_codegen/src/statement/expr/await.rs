use crate::{ByteCodegen, Res};
use swc_ecma_ast::AwaitExpr;

impl ByteCodegen {
    pub fn compile_await(&mut self, stmt: &AwaitExpr) -> Res {
        todo!()
    }
}
