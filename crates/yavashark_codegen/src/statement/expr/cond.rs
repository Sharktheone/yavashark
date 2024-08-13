use crate::{ByteCodegen, Res};
use swc_ecma_ast::CondExpr;
use yavashark_bytecode::Instruction;

impl ByteCodegen {
    pub fn compile_cond(&mut self, stmt: &CondExpr) -> Res {
        let CondExpr { test, cons, alt, .. } = stmt;
        
        self.compile_expr(test, stmt.span)?;
        
        let idx = self.instructions.len();
        self.instructions.push(Instruction::JmpIfNotAccRel(1));
        
        self.compile_expr(cons, stmt.span)?;
        
        
        let idx2 = self.instructions.len();
        self.instructions.push(Instruction::Jmp(0));
        
        self.instructions[idx] = Instruction::JmpIfNotAccRel(self.instructions.len() as isize - idx as isize);
        
        self.compile_expr(alt, stmt.span)?;
        
        self.instructions[idx2] = Instruction::JmpRel(self.instructions.len() as isize - idx2 as isize);
        
        
        Ok(())
    }
}
