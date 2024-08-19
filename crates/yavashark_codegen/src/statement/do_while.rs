use crate::{ByteCodegen, Res};
use swc_ecma_ast::{DoWhileStmt, ExprStmt};
use yavashark_bytecode::Instruction;

impl ByteCodegen {
    pub fn compile_do_while(&mut self, stmt: &DoWhileStmt) -> Res {
        let idx = self.instructions.len();

        self.compile_statement(&stmt.body)?;

        self.compile_expr(&stmt.test, stmt.span)?;

        self.instructions.push(Instruction::JmpIfAccRel(
            idx as isize - self.instructions.len() as isize,
        ));

        Ok(())
    }
}
