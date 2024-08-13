use crate::{ByteCodegen, Res};
use swc_ecma_ast::{ExprStmt, IfStmt};
use yavashark_bytecode::Instruction;

impl ByteCodegen {
    pub fn compile_if(&mut self, stmt: &IfStmt) -> Res {
        let IfStmt {
            test, cons, alt, ..
        } = stmt;

        self.compile_expr(test, stmt.span)?;

        let idx = self.instructions.len();
        self.instructions.push(Instruction::JmpIfNotAccRel(1));

        self.compile_statement(cons)?;

        if let Some(alt) = alt {
            let idx2 = self.instructions.len();
            self.instructions.push(Instruction::Jmp(0));
            self.instructions[idx] = Instruction::JmpIfNotAccRel(self.instructions.len() as isize - idx as isize);
            self.compile_statement(alt)?;
            
            self.instructions[idx2] = Instruction::JmpRel(self.instructions.len() as isize - idx2 as isize);
            
        } else {
            self.instructions[idx] = Instruction::JmpIfNotAccRel(self.instructions.len() as isize - idx as isize);
        }

        Ok(())
    }
}
