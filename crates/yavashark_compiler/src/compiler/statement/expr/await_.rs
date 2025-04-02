use crate::{Compiler, Res};
use swc_ecma_ast::AwaitExpr;
use yavashark_bytecode::data::OutputData;
use yavashark_bytecode::instructions::Instruction;

impl Compiler {
    pub fn compile_await(&mut self, expr: &AwaitExpr, out: Option<impl OutputData>) -> Res {
        let expr = self.compile_expr_data_acc(&expr.arg)?;

        if let Some(out) = out {
            self.instructions.push(Instruction::await_(expr, out));
        } else {
            self.instructions.push(Instruction::await_no_output(expr));
        }

        Ok(())
    }
}
