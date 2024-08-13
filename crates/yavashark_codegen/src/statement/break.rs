use crate::{ByteCodegen, Res};
use swc_ecma_ast::BreakStmt;
use yavashark_bytecode::Instruction;

impl ByteCodegen {
    pub fn compile_break(&mut self, stmt: &BreakStmt) -> Res {
        if let Some(label) = &stmt.label {
            let name = label.sym.to_string();
            
            self.backpatch(name, self.instructions.len());
            
            self.instructions.push(Instruction::JmpRel(1));
        } else {
            self.backpatch_break(self.instructions.len());
            
            self.instructions.push(Instruction::JmpRel(1));
        }
        
        Ok(())
    }
}
