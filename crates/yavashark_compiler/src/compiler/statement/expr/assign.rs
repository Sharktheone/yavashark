use swc_ecma_ast::AssignExpr;
use yavashark_bytecode::data::OutputData;
use crate::{Compiler, Res};

impl Compiler {
    pub fn compile_assign(&mut self, expr: &AssignExpr, out: Option<impl OutputData>) -> Res {
        todo!()
    }
}