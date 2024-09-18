use anyhow::anyhow;
use crate::{ByteCodegen, Res};
use swc_ecma_ast::{CallExpr, Callee};
use yavashark_bytecode::Instruction;

impl ByteCodegen {
    pub fn compile_call(&mut self, stmt: &CallExpr) -> Res {
        
        if stmt.args.len() > u16::MAX as usize {
            return Err(anyhow!("Too many arguments"));
        }
        
        
        let Callee::Expr(expr) = &stmt.callee else {
            todo!()
        };

        self.compile_expr(expr, stmt.span)?;
        
        
        
        for arg in &stmt.args {
            //TODO: push the args here
        }

        self.instructions.push(Instruction::CallAcc(stmt.args.len() as u16)); //TODO: how can we push the args here?

        Ok(())
    }
}
