use swc_ecma_ast::SuperPropExpr;
use yavashark_bytecode::data::OutputData;
use crate::{Compiler, Res};

impl Compiler {
    pub fn compile_super_prop(&mut self, expr: &SuperPropExpr, out: Option<impl OutputData>) -> Res {
        todo!()
    }
}