use crate::{Compiler, Res};
use swc_ecma_ast::CondExpr;
use yavashark_bytecode::data::OutputData;
use yavashark_bytecode::instructions::Instruction;
use yavashark_bytecode::jmp::Test;
use yavashark_bytecode::JmpAddr;

impl Compiler {
    pub fn compile_cond(&mut self, expr: &CondExpr, out: Option<impl OutputData>) -> Res {
        let cond = self.compile_test_expr(&expr.test)?;

        if cond == Test::Always {
            self.compile_expr(&expr.alt, out)?;
        } else if cond == Test::Never {
            self.compile_expr(&expr.cons, out)?;
        } else {
            let jmp = self.instructions.len();
            self.instructions.push(Instruction::JmpRel(0));

            self.compile_expr(&expr.cons, out)?;

            if let Some(inst) = cond.get(self.instructions.len() + 1 as JmpAddr) {
                self.instructions[jmp] = inst;
            }
            let jmp = self.instructions.len();
            self.instructions.push(Instruction::JmpRel(0));

            self.compile_expr(&expr.alt, out)?;

            if let Some(inst) = Test::Always.get(self.instructions.len() as JmpAddr) {
                self.instructions[jmp] = inst;
            }
        }

        Ok(())
    }
}
