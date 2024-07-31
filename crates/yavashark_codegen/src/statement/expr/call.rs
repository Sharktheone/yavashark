use crate::{ByteCodegen, Res};
use swc_ecma_ast::{CallExpr, Callee};
use yavashark_bytecode::Instruction;

impl ByteCodegen {
    pub fn compile_call(&mut self, stmt: &CallExpr) -> Res {
        let Callee::Expr(expr) = &stmt.callee else {
            todo!()
        };

        self.compile_expr(expr, stmt.span)?;

        self.instructions.push(Instruction::CallAcc); //TODO: how can we push the args here?

        Ok(())
    }
}
