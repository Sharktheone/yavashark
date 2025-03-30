use swc_ecma_ast::Tpl;
use yavashark_bytecode::data::OutputData;
use crate::{Compiler, Res};

impl Compiler {
    pub fn compile_tpl(&mut self, expr: &Tpl, out: Option<impl OutputData>) -> Res {
        todo!()
    }
}