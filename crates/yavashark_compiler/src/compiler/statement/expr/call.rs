use crate::node::ASTNode;
use crate::{Compiler, Res};
use swc_ecma_ast::{CallExpr, Callee, Expr};
use yavashark_bytecode::data::{Acc, OutputData};
use yavashark_bytecode::instructions::Instruction;

impl Compiler {
    pub fn compile_call(&mut self, expr: &CallExpr, out: Option<impl OutputData>) -> Res {
        self.compile_call_args(&expr.args)?;

        let callee = match &expr.callee {
            Callee::Expr(expr) => {
                if let Expr::Member(m) = &**expr {
                    let member = self.compile_member_prop(&m.prop)?;
                    let prop = self.compile_expr_data_acc(&m.obj)?;

                    if let Some(out) = out {
                        self.instructions.push(Instruction::call_member(prop, member, out));
                    } else {
                        self.instructions.push(Instruction::call_member_no_output(prop, member));
                    }

                    self.dealloc(prop);
                    self.dealloc(member);

                    return Ok(())
                }


                self.compile_expr_data_acc(expr)?
            },
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
            self.instructions.push(Instruction::call_no_output(callee));
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
