use crate::{Compiler, Res};
use swc_ecma_ast::NewExpr;
use yavashark_bytecode::data::OutputData;

impl Compiler {
    pub fn compile_new(&mut self, expr: &NewExpr, out: Option<impl OutputData>) -> Res {
        todo!()
    }
}
