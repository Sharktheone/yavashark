use crate::{ByteCodegen, Res};
use swc_ecma_ast::{ExprStmt, ForStmt, VarDeclOrExpr};
use yavashark_bytecode::Instruction;

impl ByteCodegen {
    pub fn compile_for(&mut self, stmt: &ForStmt) -> Res {
        if let Some(init) = &stmt.init {
            match init {
                VarDeclOrExpr::VarDecl(v) => {
                    // self.decl_var(v)?;
                    todo!()
                }
                VarDeclOrExpr::Expr(e) => {
                    self.compile_expr(e, stmt.span)?;
                }
            }
        }
        
        let idx = self.instructions.len();
        let mut idx2 = None;
        
        if let Some(test) = &stmt.test {
            self.compile_expr(test, stmt.span)?;
            
            idx2 = Some(self.instructions.len());
            self.instructions.push(Instruction::JmpIfNotAccRel(1));
        }
        
        self.compile_statement(&stmt.body)?;
        
        if let Some(update) = &stmt.update {
            self.compile_expr(update, stmt.span)?;
        }
        
        self.instructions.push(Instruction::JmpRel(idx as isize - self.instructions.len() as isize));
        
        if let Some(idx2) = idx2 {
            self.instructions[idx2] = Instruction::JmpIfNotAccRel(self.instructions.len() as isize - idx2 as isize);
        }
        
        Ok(())
    }
}
