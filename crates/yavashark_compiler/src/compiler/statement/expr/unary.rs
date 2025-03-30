use swc_ecma_ast::UnaryExpr;
use yavashark_bytecode::data::OutputData;
use crate::{Compiler, Res};

impl Compiler {
    pub fn compile_unary(&mut self, expr: &UnaryExpr, out: Option<impl OutputData>) -> Res {
        todo!()
    }
}