use crate::{Compiler, Res};
use swc_ecma_ast::DoWhileStmt;
use yavashark_bytecode::instructions::Instruction;
use yavashark_bytecode::jmp::Test;

impl Compiler {
    pub fn compile_do_while(&mut self, d: &DoWhileStmt) -> Res {
        let start = self.instructions.len();

        self.compile_stmt(&d.body)?;

        let cond = self.compile_test_expr(&d.test)?;

        match cond {
            Test::Always => {
                self.instructions.push(Instruction::jmp(start));
            }
            Test::Never => {},
            _ => {
                if let Some(inst) = cond.get(start as yavashark_bytecode::JmpAddr) {
                    self.instructions.push(inst);
                }
            }
        }

        Ok(())
    }
}