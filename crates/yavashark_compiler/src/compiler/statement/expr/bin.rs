use crate::{Compiler, Res};
use swc_ecma_ast::BinExpr;
use yavashark_bytecode::data::OutputData;

impl Compiler {
    pub fn compile_bin(&mut self, expr: &BinExpr, out: Option<impl OutputData>) -> Res {
        todo!()
    }
}
