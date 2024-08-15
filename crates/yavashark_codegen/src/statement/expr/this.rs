use crate::{ByteCodegen, Res};
use swc_ecma_ast::ThisExpr;
use yavashark_bytecode::Instruction;

impl ByteCodegen {
    pub fn compile_this(&mut self, stmt: &ThisExpr) -> Res {
        self.instructions.push(Instruction::ThisAcc):

        Ok(())
    }
}
