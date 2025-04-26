use crate::{Compiler, Res};
use swc_ecma_ast::{ForStmt, VarDeclOrExpr};
use yavashark_bytecode::{JmpAddr, instructions::Instruction, jmp::Test};

impl Compiler {
    pub fn compile_for(&mut self, f: &ForStmt) -> Res {
        if let Some(init) = &f.init {
            match init {
                VarDeclOrExpr::VarDecl(vd) => self.decl_var(vd)?,
                VarDeclOrExpr::Expr(expr) => {
                    self.compile_expr_no_out(expr)?;
                }
            }
        }

        let start = self.instructions.len();

        let cond = if let Some(test_expr) = &f.test {
            self.compile_test_expr(test_expr)?
        } else {
            Test::Always
        };

        if cond == Test::Always {
            self.compile_stmt(&f.body)?;
            if let Some(update_expr) = &f.update {
                self.compile_expr_no_out(update_expr)?;
            }
            self.instructions.push(Instruction::jmp(start));
        } else if cond != Test::Never {
            let jmp_pos = self.instructions.len();
            self.instructions.push(Instruction::JmpRel(0));

            self.compile_stmt(&f.body)?;
            if let Some(update_expr) = &f.update {
                self.compile_expr_no_out(update_expr)?;
            }
            self.instructions.push(Instruction::jmp(start));

            if let Some(inst) = cond.get(self.instructions.len() as JmpAddr) {
                self.instructions[jmp_pos] = inst;
            }
        }

        Ok(())
    }
}
