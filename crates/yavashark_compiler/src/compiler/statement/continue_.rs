use anyhow::anyhow;
use yavashark_bytecode::instructions::Instruction;
use crate::{Compiler, Res};

impl Compiler {
    pub fn compile_continue(&mut self, c: &swc_ecma_ast::ContinueStmt) -> Res {
        if let Some(label) = &c.label {
            let label = label.sym.as_str();
            let label = self.get_label(label).ok_or(anyhow!("Label not found"))?;
            
            self.instructions.push(Instruction::ContinueLabel(label));
        } else {
            self.instructions.push(Instruction::Continue);
        }
        
        Ok(())
    }
}