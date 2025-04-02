use crate::{Compiler, Res};
use swc_ecma_ast::CondExpr;
use yavashark_bytecode::data::OutputData;

impl Compiler {
    pub fn compile_cond(&mut self, expr: &CondExpr, out: Option<impl OutputData>) -> Res {
        todo!()
    }
}
