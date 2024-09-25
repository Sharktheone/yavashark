use crate::{ByteCodegen, Res};
use anyhow::anyhow;
use swc_ecma_ast::{ContinueStmt, ExprStmt};
use yavashark_bytecode::Instruction;

impl ByteCodegen {
    pub fn compile_continue(&mut self, stmt: &ContinueStmt) -> Res {
        if let Some(label) = &stmt.label {
            let name = label.sym.to_string();

            self.compile_label_jump(&name)?;
        } else {
            let target = self
                .loop_label
                .ok_or(anyhow!("Illegal continue statement"))?;

            self.instructions.push(Instruction::Jmp(target));
        }
        Ok(())
    }
}
