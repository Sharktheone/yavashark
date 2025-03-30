use swc_ecma_ast::NewExpr;
use yavashark_bytecode::data::OutputData;
use crate::{Compiler, Res};

impl Compiler {
    pub fn compile_new(&mut self, expr: &NewExpr, out: Option<impl OutputData>) -> Res {
        todo!()
    }
}