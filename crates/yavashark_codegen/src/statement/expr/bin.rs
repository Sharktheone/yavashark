use crate::{ByteCodegen, Res};
use swc_ecma_ast::{BinExpr, BinaryOp, ThisExpr};
use yavashark_bytecode::Instruction;

impl ByteCodegen {
    pub fn compile_bin(&mut self, stmt: &BinExpr) -> Res {
        self.compile_expr(&stmt.right, stmt.span)?;
        self.instructions.push(Instruction::PushAcc);

        self.compile_expr(&stmt.left, stmt.span)?;

        self.instructions.push(Instruction::PopToReg(0));

        match stmt.op {
            BinaryOp::Add => self.instructions.push(Instruction::AddAccReg(0)),
            BinaryOp::Sub => self.instructions.push(Instruction::SubAccReg(0)),
            BinaryOp::Mul => self.instructions.push(Instruction::MulAccReg(0)),
            BinaryOp::Div => self.instructions.push(Instruction::DivAccReg(0)),
            BinaryOp::Mod => self.instructions.push(Instruction::ModAccReg(0)),

            BinaryOp::BitOr => self.instructions.push(Instruction::BitOrAcc(0)),
            BinaryOp::BitXor => self.instructions.push(Instruction::BitXorAcc(0)),
            BinaryOp::BitAnd => self.instructions.push(Instruction::BitAndAcc(0)),

            BinaryOp::EqEq => self.instructions.push(Instruction::EqAcc(0)),
            BinaryOp::NotEq => self.instructions.push(Instruction::NotEqAcc(0)),
            BinaryOp::EqEqEq => self.instructions.push(Instruction::EqEqAcc(0)),
            BinaryOp::NotEqEq => self.instructions.push(Instruction::NotEqEqAcc(0)),

            BinaryOp::Lt => self.instructions.push(Instruction::LtAcc(0)),
            BinaryOp::LtEq => self.instructions.push(Instruction::LtEqAcc(0)),

            BinaryOp::Gt => self.instructions.push(Instruction::GtAcc(0)),
            BinaryOp::GtEq => self.instructions.push(Instruction::GtEqAcc(0)),

            BinaryOp::LShift => self.instructions.push(Instruction::LShiftAcc(0)),
            BinaryOp::RShift => self.instructions.push(Instruction::RShiftAcc(0)),
            BinaryOp::ZeroFillRShift => self.instructions.push(Instruction::ZeroFillRShiftAcc(0)),

            BinaryOp::LogicalOr => self.instructions.push(Instruction::LOrAcc(0)),
            BinaryOp::LogicalAnd => self.instructions.push(Instruction::LAndAcc(0)),

            BinaryOp::In => self.instructions.push(Instruction::InAcc(0)),
            BinaryOp::InstanceOf => self.instructions.push(Instruction::InstanceOfAcc(0)),

            BinaryOp::Exp => self.instructions.push(Instruction::ExpAcc(0)),

            BinaryOp::NullishCoalescing => {
                self.instructions.push(Instruction::NullishCoalescingAcc(0));
            }
        }

        Ok(())
    }
}
