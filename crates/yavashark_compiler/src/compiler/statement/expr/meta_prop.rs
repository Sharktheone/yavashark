use crate::{Compiler, Res};
use swc_ecma_ast::MetaPropExpr;
use yavashark_bytecode::data::OutputData;

impl Compiler {
    pub fn compile_meta_prop(&mut self, expr: &MetaPropExpr, out: Option<impl OutputData>) -> Res {
        todo!()
    }
}
