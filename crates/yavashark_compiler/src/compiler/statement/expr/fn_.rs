use super::MoveOptimization;
use crate::{Compiler, Res};
use swc_ecma_ast::FnExpr;
use yavashark_bytecode::data::OutputData;

impl Compiler {
    pub fn compile_fn(
        &mut self,
        expr: &FnExpr,
        out: Option<impl OutputData>,
    ) -> Res<Option<MoveOptimization>> {
        todo!()
    }
}
