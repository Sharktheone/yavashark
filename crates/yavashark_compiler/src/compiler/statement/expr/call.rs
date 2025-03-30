use swc_ecma_ast::CallExpr;
use yavashark_bytecode::data::OutputData;
use crate::{Compiler, Res};

impl Compiler {
    pub fn compile_call(&mut self, expr: &CallExpr, out: Option<impl OutputData>) -> Res {
        todo!()
    }
}