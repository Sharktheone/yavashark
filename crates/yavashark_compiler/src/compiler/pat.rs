use std::path::Component;
use swc_ecma_ast::Pat;
use yavashark_bytecode::data::Data;
use crate::{Compiler, Res};

impl Compiler {
    pub fn compile_pat(&mut self, pat: &Pat, source: impl Data) -> Res {
        todo!()
    }
}