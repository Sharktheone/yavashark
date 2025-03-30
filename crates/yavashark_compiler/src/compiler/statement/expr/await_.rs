use swc_ecma_ast::AwaitExpr;
use yavashark_bytecode::data::OutputData;
use crate::{Compiler, Res};

impl Compiler {
    pub fn compile_await(&mut self, expr: &AwaitExpr, out: Option<impl OutputData>) -> Res {
        todo!()
    }
}