use crate::{Compiler, Res};
use yavashark_bytecode::instructions::Instruction;

impl Compiler {
    pub fn compile_throw(&mut self, throw: &swc_ecma_ast::ThrowStmt) -> Res {
        let val = self.compile_expr_data_acc(&throw.arg)?;

        self.instructions.push(Instruction::throw(val));

        Ok(())
    }
}
