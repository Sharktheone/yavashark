use crate::{Compiler, Res};
use swc_ecma_ast::TaggedTpl;
use yavashark_bytecode::data::OutputData;

impl Compiler {
    pub fn compile_tagged_tpl(&mut self, expr: &TaggedTpl, out: Option<impl OutputData>) -> Res {
        todo!()
    }
}
