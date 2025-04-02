use crate::{Compiler, Res};
use yavashark_bytecode::JmpAddr;
use yavashark_bytecode::instructions::Instruction;
use yavashark_bytecode::jmp::Test;

impl Compiler {
    pub fn compile_if(&mut self, i: &swc_ecma_ast::IfStmt) -> Res {
        let cond = self.compile_test_expr(&i.test)?;

        if cond == Test::Always {
            if let Some(alt) = &i.alt {
                self.compile_stmt(alt)?;
            }
        } else if cond == Test::Never {
            self.compile_stmt(&i.cons)?;
        } else {
            let jmp = self.instructions.len();
            self.instructions.push(Instruction::JmpRel(0));

            self.compile_stmt(&i.cons)?;

            if let Some(alt) = &i.alt {
                if let Some(inst) = cond.get(self.instructions.len() + 1 as JmpAddr) {
                    self.instructions[jmp] = inst;
                }
                let jmp = self.instructions.len();
                self.instructions.push(Instruction::JmpRel(0));

                self.compile_stmt(alt)?;

                if let Some(inst) = Test::Always.get(self.instructions.len() as JmpAddr) {
                    self.instructions[jmp] = inst;
                }
            } else if let Some(inst) = cond.get(self.instructions.len() as JmpAddr) {
                self.instructions[jmp] = inst;
            }
        }

        Ok(())
    }
}
