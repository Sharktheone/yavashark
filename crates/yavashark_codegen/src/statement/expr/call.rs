use anyhow::anyhow;
use swc_common::Spanned;
use crate::{ByteCodegen, Res};
use swc_ecma_ast::{CallExpr, Callee};
use yavashark_bytecode::Instruction;

impl ByteCodegen {
    pub fn compile_call(&mut self, stmt: &CallExpr) -> Res {
        
        if stmt.args.len() > u16::MAX as usize {
            return Err(anyhow!("Too many arguments"));
        }

        for arg in &stmt.args {
            self.compile_expr(&arg.expr, arg.expr.span())?;

            self.instructions.push(Instruction::PushAcc);
        }
        
        let Callee::Expr(expr) = &stmt.callee else {
            todo!()
        };

        self.compile_expr(expr, stmt.span)?;
        
        
        

        self.instructions.push(Instruction::CallAcc(stmt.args.len() as u16)); //TODO: how can we push the args here?
        //TODO: variadic args?

        Ok(())
    }
}
