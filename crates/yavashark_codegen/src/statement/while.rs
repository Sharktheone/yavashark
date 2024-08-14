use crate::{ByteCodegen, Res};
use swc_ecma_ast::{ExprStmt, WhileStmt};
use yavashark_bytecode::Instruction;

impl ByteCodegen {
    pub fn compile_while(&mut self, stmt: &WhileStmt) -> Res {
        let idx = self.instructions.len();

        self.compile_expr(&stmt.test, stmt.span)?;

        let idx2 = self.instructions.len();
        self.instructions.push(Instruction::JmpIfNotAccRel(1));

        self.compile_statement(&stmt.body)?;

        self.instructions.push(Instruction::JmpRel(idx as isize - self.instructions.len() as isize));

        self.instructions[idx2] = Instruction::JmpIfNotAccRel(self.instructions.len() as isize - idx2 as isize);

        Ok(())

    }
}
