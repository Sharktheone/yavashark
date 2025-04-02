use crate::{Compiler, Res};
use swc_ecma_ast::YieldExpr;
use yavashark_bytecode::data::OutputData;

impl Compiler {
    pub fn compile_yield(&mut self, expr: &YieldExpr, out: Option<impl OutputData>) -> Res {
        todo!()
    }
}
