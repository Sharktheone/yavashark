use swc_ecma_ast::ArrayLit;
use yavashark_bytecode::data::OutputData;
use crate::{Compiler, Res};
use super::MoveOptimization;

impl Compiler {
    pub fn compile_array(&mut self, expr: &ArrayLit, out: Option<impl OutputData>) -> Res<Option<MoveOptimization>> {
        todo!()
    }
}