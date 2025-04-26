use crate::{Compiler, Res};
use swc_ecma_ast::WhileStmt;
use yavashark_bytecode::instructions::Instruction;
use yavashark_bytecode::jmp::Test;

impl Compiler {
    pub fn compile_while(&mut self, w: &WhileStmt) -> Res {
        let addr = self.instructions.len();
        let cond = self.compile_test_expr(&w.test)?;

        if cond == Test::Always {
            self.compile_stmt(&w.body)?;
            self.instructions.push(Instruction::jmp(addr));
        } else if cond == Test::Never {
            return Ok(());
        } else {
            let jmp = self.instructions.len();
            self.instructions.push(Instruction::JmpRel(0));

            self.compile_stmt(&w.body)?;

            self.instructions.push(Instruction::jmp(addr));

            if let Some(inst) = cond.get(self.instructions.len() as yavashark_bytecode::JmpAddr) {
                self.instructions[jmp] = inst;
            }
        }

        Ok(())
    }
}
