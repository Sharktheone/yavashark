use crate::compiler::statement::expr::MoveOptimization;
use crate::{Compiler, Res};
use swc_ecma_ast::ClassExpr;
use yavashark_bytecode::data::OutputData;

impl Compiler {
    pub fn compile_class(
        &mut self,
        expr: &ClassExpr,
        out: Option<impl OutputData>,
    ) -> Res<Option<MoveOptimization>> {
        todo!()
    }
}
