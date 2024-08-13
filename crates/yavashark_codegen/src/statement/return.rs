use crate::{ByteCodegen, Res};
use swc_ecma_ast::{ExprStmt, ReturnStmt};

impl ByteCodegen {
    pub fn compile_return(&mut self, stmt: &ReturnStmt) -> Res {
        if let Some(arg) = &stmt.arg {
            self.compile_expr(arg, stmt.span)?;

            self.instructions
                .push(yavashark_bytecode::Instruction::ReturnAcc);
        } else {
            self.instructions
                .push(yavashark_bytecode::Instruction::Return);
        }

        Ok(())
    }
}
