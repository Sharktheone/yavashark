use crate::Compiler;
use yavashark_bytecode::data::Acc;
use yavashark_bytecode::jmp::Test;

impl Compiler {
    pub fn test_seq(&mut self, expr: &swc_ecma_ast::SeqExpr) -> crate::Res<Test> {
        if expr.exprs.is_empty() {
            return Ok(Test::Never);
        }

        if expr.exprs.len() == 1 {
            return self.compile_test_expr(&expr.exprs[0]);
        }

        for expr in &expr.exprs[..expr.exprs.len() - 1] {
            self.compile_expr(expr, None::<Acc>)?;
        }

        self.compile_test_expr(&expr.exprs[expr.exprs.len() - 1])
    }
}
