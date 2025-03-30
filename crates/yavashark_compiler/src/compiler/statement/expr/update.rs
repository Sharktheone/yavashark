use swc_ecma_ast::UpdateExpr;
use yavashark_bytecode::data::OutputData;
use crate::{Compiler, Res};

impl Compiler {
    pub fn compile_update(&mut self, expr: &UpdateExpr, out: Option<impl OutputData>) -> Res {
        todo!()
    }
}