use swc_ecma_ast::ArrowExpr;
use yavashark_bytecode::data::OutputData;
use crate::{Compiler, Res};

impl Compiler {
    pub fn compile_arrow(&mut self, expr: &ArrowExpr, out: Option<impl OutputData>) -> Res {
        todo!()
    }
}