use swc_ecma_ast::BinExpr;
use yavashark_bytecode::data::OutputData;
use crate::{Compiler, Res};

impl Compiler {
    pub fn compile_bin(&mut self, expr: &BinExpr, out: Option<impl OutputData>) -> Res {
        todo!()
    }
}