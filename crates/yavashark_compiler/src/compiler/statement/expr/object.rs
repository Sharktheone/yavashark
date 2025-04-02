use super::MoveOptimization;
use crate::{Compiler, Res};
use swc_ecma_ast::ObjectLit;
use yavashark_bytecode::data::OutputData;

impl Compiler {
    pub fn compile_object(
        &mut self,
        expr: &ObjectLit,
        out: Option<impl OutputData>,
    ) -> Res<Option<MoveOptimization>> {
        todo!()
    }
}
