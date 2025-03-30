use swc_ecma_ast::ClassExpr;
use yavashark_bytecode::data::OutputData;
use crate::{Compiler, Res};
use crate::compiler::statement::expr::MoveOptimization;

impl Compiler {
    pub fn compile_class(&mut self, expr: &ClassExpr, out: Option<impl OutputData>) -> Res<Option<MoveOptimization>> {
        todo!()
    }
}