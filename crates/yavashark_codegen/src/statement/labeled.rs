use std::fmt::format;
use crate::{ByteCodegen, Res};
use swc_ecma_ast::{ExprStmt, LabeledStmt, ReturnStmt};
use log::debug;
use yavashark_bytecode::Instruction;

impl ByteCodegen {
    pub fn compile_labeled(&mut self, stmt: &LabeledStmt) -> Res {
        
        
        let name = stmt.label.sym.to_string();
        
        self.labels.push((name.clone(), self.instructions.len()));

        self.compile_stmt(&stmt.body)?;

        let lbl = self.labels.pop();
        
        debug_assert_eq!(lbl.unwrap().0, name.clone(), "Label mismatch; coding error");
        
        
        self.backpatch_label(name, self.instructions.len());
        
        Ok(())
    }
    
    
    fn backpatch_label(&mut self, name: String, pos: usize) {
        let (backpatch, rem) = self.label_backpatch.drain(..).partition(|(n, _)| n == &name);
        
        self.label_backpatch = rem;
        
        
        for (n, p) in backpatch {
            
            self.instructions[p] = Instruction::Jmp(pos);
        }
    }
}
