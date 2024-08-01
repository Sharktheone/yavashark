use crate::{ByteCodegen, Res};
use swc_ecma_ast::{ExprStmt, IfStmt};
use yavashark_bytecode::Instruction;

impl ByteCodegen {
    pub fn compile_if(&mut self, stmt: &IfStmt) -> Res {
        let IfStmt {
            test,
            cons,
            alt,
            ..
        } = stmt;

        self.compile_expr(test, stmt.span)?;

        let idx = self.instructions.len();
        self.instructions.push(Instruction::JmpIfNotAcc(0));

        self.compile_statement(cons)?;
        

        if let Some(alt) = alt {
            self.instructions.push(Instruction::Jmp(0));
            self.instructions[idx] = Instruction::JmpIfNotAcc(self.instructions.len() as i32);
            self.compile_statement(alt)?;
        } else {
            self.instructions[idx] = Instruction::JmpIfNotAcc(self.instructions.len() as i32);
        }

        Ok(())
    }
}
