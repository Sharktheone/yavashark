use crate::{Compiler, Res};
use swc_ecma_ast::{UnaryExpr, UnaryOp};
use yavashark_bytecode::data::OutputData;
use yavashark_bytecode::instructions::Instruction;

impl Compiler {
    pub fn compile_unary(&mut self, expr: &UnaryExpr, out: Option<impl OutputData>) -> Res {
        match expr.op {
            UnaryOp::Void => {
                self.compile_expr_no_out(&expr.arg)?;
                Ok(())
            }
            UnaryOp::TypeOf => {
                let data = self.compile_expr_data_acc(&expr.arg)?;

                if let Some(out) = out {
                    self.instructions.push(Instruction::type_of(data, out));
                }

                self.dealloc(data);

                Ok(())
            }
            UnaryOp::Delete => {
                todo!()
            }
            UnaryOp::Plus => {
                let data = self.compile_expr_data_acc(&expr.arg)?;

                if let Some(out) = out {
                    self.instructions.push(Instruction::to_number(data, out));
                }

                self.dealloc(data);

                Ok(())
            }
            UnaryOp::Minus => {
                let data = self.compile_expr_data_acc(&expr.arg)?;

                if let Some(out) = out {
                    self.instructions.push(Instruction::negate(data, out));
                }

                self.dealloc(data);

                Ok(())
            }
            UnaryOp::Bang => {
                let data = self.compile_expr_data_acc(&expr.arg)?;
                if let Some(out) = out {
                    self.instructions.push(Instruction::logical_not(data, out));
                }
                Ok(())
            }
            UnaryOp::Tilde => {
                let data = self.compile_expr_data_acc(&expr.arg)?;

                if let Some(out) = out {
                    self.instructions.push(Instruction::bitwise_not(data, out));
                }

                self.dealloc(data);

                Ok(())
            }
        }
    }
}
