use crate::{Compiler, Res};
use swc_ecma_ast::{AssignExpr, AssignOp, AssignTarget, SimpleAssignTarget};
use yavashark_bytecode::data::{Acc, Data, OutputData, OutputDataType};
use yavashark_bytecode::instructions::Instruction;

impl Compiler {
    pub fn compile_assign(&mut self, expr: &AssignExpr, output: Option<impl OutputData>) -> Res {
        let x = match expr.op {
            AssignOp::Assign => Instruction::move_,
            AssignOp::AddAssign => Instruction::add_assign,
            AssignOp::SubAssign => Instruction::sub_assign,
            AssignOp::MulAssign => Instruction::mul_assign,
            AssignOp::DivAssign => Instruction::div_assign,
            AssignOp::ModAssign => Instruction::rem_assign,
            AssignOp::LShiftAssign => Instruction::l_shift_assign,
            AssignOp::RShiftAssign => Instruction::r_shift_assign,
            AssignOp::ZeroFillRShiftAssign => Instruction::zero_fill_r_shift_assign,
            AssignOp::BitAndAssign => Instruction::b_and_assign,
            AssignOp::BitOrAssign => Instruction::b_or_assign,
            AssignOp::BitXorAssign => Instruction::b_xor_assign,
            AssignOp::ExpAssign => Instruction::exp_assign,
            AssignOp::AndAssign => Instruction::and_assign,
            AssignOp::OrAssign => Instruction::or_assign,
            AssignOp::NullishAssign => Instruction::nullish_assign,
        };

        let val_ = self.alloc_reg_or_stack();
        let val = self.compile_expr_data(&expr.right, Some(val_))?;

        let out = match &expr.left {
            AssignTarget::Simple(simple) => {
                match simple {
                    SimpleAssignTarget::Ident(ident) => {
                        OutputData::data_type(self.get_ident(&ident.id))
                    }
                    SimpleAssignTarget::Member(m) => {
                        self.compile_member(m, Some(Acc))?;
                        OutputData::data_type(Acc) //TODO: we need to update the member again...
                    }
                    _ => todo!(),
                }
            }
            AssignTarget::Pat(_) => {
                todo!()
            }
        };

        self.instructions.push(x(val, out));

        if let Some(output) = output {
            if out != OutputDataType::Acc(Acc) {
                self.instructions.push(Instruction::move_(out, output));
            }
        }

        self.dealloc(val);
        self.dealloc(val_);

        Ok(())
    }
}
