use crate::{Compiler, Res};
use anyhow::bail;
use swc_ecma_ast::{Expr, SimpleAssignTarget, UpdateExpr, UpdateOp};
use yavashark_bytecode::data::OutputData;
use yavashark_bytecode::instructions::Instruction;

impl Compiler {
    pub fn compile_update(&mut self, expr: &UpdateExpr, out: Option<impl OutputData>) -> Res {
        let op = match expr.op {
            UpdateOp::PlusPlus => Instruction::inc,
            UpdateOp::MinusMinus => Instruction::dec,
        };

        let (source, member) = match &*expr.arg {
            Expr::Ident(ident) => (self.alloc_var(ident.sym.as_str()).data_type(), None),
            Expr::Member(member) => {
                let m = self.compile_member_prop(&member.prop)?;
                let prop = self.compile_expr_data_acc(&member.obj)?;
                let loc = self.alloc_reg_or_stack();

                self.instructions
                    .push(Instruction::load_member(prop, m, loc));

                (loc.data_type(), Some((m, prop)))
            }

            _ => bail!("Invalid left-hand side expression in update expression"),
        };

        if let Some(out) = out {
            if !expr.prefix {
                self.instructions.push(Instruction::move_(source, out)); //TODO: this is incorrect, since we need to also convert the value to a number
            }
        }

        self.instructions.push(op(source, source));

        if let Some(out) = out {
            if expr.prefix {
                self.instructions.push(Instruction::move_(source, out));
            }
        }

        if let Some((m, prop)) = member {
            //TODO: store member

            // self.instructions
            //     .push(Instruction::store_member(prop, m, source));

            self.dealloc(prop);
            self.dealloc(m);
        }

        self.dealloc(source);

        Ok(())
    }
}
