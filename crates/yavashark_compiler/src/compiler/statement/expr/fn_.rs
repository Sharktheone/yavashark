use swc_ecma_ast::FnExpr;
use yavashark_bytecode::data::OutputData;
use crate::{Compiler, Res};
use super::MoveOptimization;

impl Compiler {
    pub fn compile_fn(&mut self, expr: &FnExpr, out: Option<impl OutputData>) -> Res<Option<MoveOptimization>> {
        todo!()
    }
}