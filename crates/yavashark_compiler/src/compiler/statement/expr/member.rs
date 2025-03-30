use swc_ecma_ast::MemberExpr;
use yavashark_bytecode::data::OutputData;
use crate::{Compiler, Res};

impl Compiler {
    pub fn compile_member(&mut self, expr: &MemberExpr, out: Option<impl OutputData>) -> Res {
        todo!()
    }
}