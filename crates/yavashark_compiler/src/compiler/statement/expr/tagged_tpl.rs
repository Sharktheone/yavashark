use swc_ecma_ast::TaggedTpl;
use yavashark_bytecode::data::OutputData;
use crate::{Compiler, Res};

impl Compiler {
    pub fn compile_tagged_tpl(&mut self, expr: &TaggedTpl, out: Option<impl OutputData>) -> Res {
        todo!()
    }
}