use crate::{Compiler, Res};
use std::path::Component;
use swc_ecma_ast::Pat;
use yavashark_bytecode::data::Data;

impl Compiler {
    pub fn compile_pat(&mut self, pat: &Pat, source: impl Data) -> Res {
        todo!()
    }
}
