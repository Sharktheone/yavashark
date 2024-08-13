use crate::{ByteCodegen, Res};
use swc_ecma_ast::ParenExpr;

impl ByteCodegen {
    pub fn compile_paren(&mut self, stmt: &ParenExpr) -> Res {
        self.compile_expr(&stmt.expr, stmt.span)
    }
}
