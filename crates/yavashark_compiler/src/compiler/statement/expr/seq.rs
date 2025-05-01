use crate::compiler::statement::expr::MoveOptimization;
use crate::{Compiler, Res};
use swc_ecma_ast::SeqExpr;
use yavashark_bytecode::data::{Acc, OutputData};

impl Compiler {
    pub fn compile_seq(
        &mut self,
        expr: &SeqExpr,
        out: Option<impl OutputData>,
    ) -> Res<Option<MoveOptimization>> {
        if expr.exprs.is_empty() {
            return Ok(None);
        }

        if expr.exprs.len() == 1 {
            return self.compile_expr(&expr.exprs[0], out);
        }

        for expr in &expr.exprs[..expr.exprs.len() - 1] {
            self.compile_expr(expr, None::<Acc>)?;
        }

        self.compile_expr(&expr.exprs[expr.exprs.len() - 1], out)
    }
}
