use crate::{Compiler, Res};
use swc_ecma_ast::YieldExpr;
use yavashark_bytecode::data::OutputData;
use yavashark_bytecode::instructions::Instruction;

impl Compiler {
    pub fn compile_yield(&mut self, expr: &YieldExpr, out: Option<impl OutputData>) -> Res {
        if let Some(arg) = &expr.arg {
            let arg = self.compile_expr_data_acc(arg)?;

            self.instructions.push(Instruction::yield_(arg));
        }

        Ok(())
    }
}
