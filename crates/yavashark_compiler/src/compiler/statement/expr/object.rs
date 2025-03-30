use swc_ecma_ast::ObjectLit;
use yavashark_bytecode::data::OutputData;
use crate::{Compiler, Res};
use super::MoveOptimization;

impl Compiler {
    pub fn compile_object(&mut self, expr: &ObjectLit, out: Option<impl OutputData>) -> Res<Option<MoveOptimization>> {
        todo!()
    }
}