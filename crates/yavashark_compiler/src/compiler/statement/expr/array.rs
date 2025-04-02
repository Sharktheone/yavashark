use super::MoveOptimization;
use crate::{Compiler, Res};
use swc_ecma_ast::ArrayLit;
use yavashark_bytecode::data::OutputData;

impl Compiler {
    pub fn compile_array(
        &mut self,
        expr: &ArrayLit,
        out: Option<impl OutputData>,
    ) -> Res<Option<MoveOptimization>> {
        todo!()
    }
}
