use crate::node::ASTNode;
use crate::{Compiler, Res};
use swc_ecma_ast::{CallExpr, Callee};
use yavashark_bytecode::data::{Acc, OutputData};
use yavashark_bytecode::instructions::Instruction;

impl Compiler {
    pub fn compile_call(&mut self, expr: &CallExpr, out: Option<impl OutputData>) -> Res {
        self.compile_call_args(&expr.args)?;

        let callee = match &expr.callee {
            Callee::Expr(expr) => self.compile_expr_data_acc(expr)?,
            Callee::Super(_) => {
                // self.instructions.push(Instruction::load_super(Acc));
                todo!()
            }
            Callee::Import(_) => {
                todo!()
            }
        };

        if let Some(out) = out {
            self.instructions.push(Instruction::call(callee, out));
        } else {
            self.instructions.push(Instruction::call(callee, Acc)); //TODO: call_no_output
        }

        Ok(())
    }

    pub fn compile_call_args(&mut self, args: &[swc_ecma_ast::ExprOrSpread]) -> Res {
        let args_have_call = args.iter().any(|arg| arg.expr.has_call());

        if args_have_call {
            todo!()
        } else {
            for arg in args {
                let out = self.compile_expr_data_acc(&arg.expr)?;

                if arg.spread.is_some() {
                    self.instructions.push(Instruction::spread_call(out));
                } else {
                    self.instructions.push(Instruction::push_call(out));
                }
            }
        }

        Ok(())
    }
}
