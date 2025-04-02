use crate::{Compiler, Res};
use swc_ecma_ast::ParenExpr;
use yavashark_bytecode::jmp::Test;

impl Compiler {
    pub fn test_paren(&mut self, expr: &ParenExpr) -> Res<Test> {
        self.compile_test_expr(&expr.expr)
    }
}
