use swc_ecma_ast::ParenExpr;
use yavashark_bytecode::data::OutputData;
use crate::{Compiler, Res};
use crate::compiler::statement::expr::MoveOptimization;

impl Compiler {
    pub fn compile_paren(&mut self, expr: &ParenExpr, out: Option<impl OutputData>) -> Res<Option<MoveOptimization>> {
        self.compile_expr(&expr.expr, out)
    }
}