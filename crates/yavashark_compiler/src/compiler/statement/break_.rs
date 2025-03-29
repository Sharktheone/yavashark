use anyhow::anyhow;
use yavashark_bytecode::instructions::Instruction;
use crate::{Compiler, Res};

impl Compiler {
    pub fn compile_break(&mut self, brk: &swc_ecma_ast::BreakStmt) -> Res {
        if let Some(label) = &brk.label {
            let label = self.get_label(&label.sym.as_str()).ok_or(anyhow!("Label not found"))?;
            
            self.instructions.push(Instruction::BreakLabel(label));
        } else {
            self.instructions.push(Instruction::Break);
        }
        
        Ok(())
    }
}