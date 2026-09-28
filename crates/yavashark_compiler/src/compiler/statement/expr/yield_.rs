use crate::{Compiler, Res};
use swc_ecma_ast::YieldExpr;
use yavashark_bytecode::data::{Acc, OutputData, Undefined};
use yavashark_bytecode::instructions::Instruction;

impl Compiler {
    pub fn compile_yield(&mut self, expr: &YieldExpr, out: Option<impl OutputData>) -> Res {
        if let Some(arg) = &expr.arg {
            let arg = self.compile_expr_data_acc(arg)?;

            if expr.delegate {
                self.instructions.push(Instruction::yield_star(arg));
            } else {
                self.instructions.push(Instruction::yield_(arg));
            }
        } else if expr.delegate {
            self.instructions.push(Instruction::yield_star(Undefined));
        } else {
            self.instructions.push(Instruction::yield_no_value());
        }

        if let Some(out) = out {
            self.instructions.push(Instruction::move_(Acc, out));
        }
        Ok(())
    }
}
