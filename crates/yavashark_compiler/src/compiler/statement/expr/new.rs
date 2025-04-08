use crate::{Compiler, Res};
use swc_ecma_ast::NewExpr;
use yavashark_bytecode::data::OutputData;
use yavashark_bytecode::instructions::Instruction;

impl Compiler {
    pub fn compile_new(&mut self, expr: &NewExpr, out: Option<impl OutputData>) -> Res {
        if let Some(args) = &expr.args {
            self.compile_call_args(args)?;
        }

        let callee = self.compile_expr_data_acc(&expr.callee)?;

        match out {
            Some(out) => {
                self.instructions.push(Instruction::construct(callee, out));
            }
            None => {
                self.instructions
                    .push(Instruction::construct_no_output(callee));
            }
        }

        Ok(())
    }
}
