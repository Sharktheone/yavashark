use crate::{Compiler, Res};
use swc_ecma_ast::SeqExpr;
use yavashark_bytecode::data::OutputData;

impl Compiler {
    pub fn compile_seq(&mut self, expr: &SeqExpr, out: Option<impl OutputData>) -> Res {
        todo!()
    }
}
