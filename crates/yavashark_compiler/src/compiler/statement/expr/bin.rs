use crate::{Compiler, Res};
use swc_ecma_ast::{BinExpr, BinaryOp};
use yavashark_bytecode::data::{DataType, OutputData};
use yavashark_bytecode::instructions::Instruction;

impl Compiler {
    pub fn compile_bin(&mut self, expr: &BinExpr, out: Option<impl OutputData>) -> Res {
        let Some(out) = out else {
            return Ok(())
        };
        
        let reg1 = self.alloc_reg_or_stack();
        let reg2 = self.alloc_reg_or_stack();
        
        //TODO: in theory we can optimize some things out here...
        let left = self.compile_expr_data(&expr.left, Some(reg1))?;
        let right = self.compile_expr_data(&expr.right, Some(reg2))?;
        
        self.instructions.push(
            get_bin_instruction(
                left,
                right,
                out,
                expr.op,
            )
        );
        
        self.dealloc(reg1);
        self.dealloc(reg2);
        self.dealloc(left);
        self.dealloc(right);
        
        Ok(())
    }
    
}

pub fn get_bin_instruction(left: DataType, right: DataType, out: impl OutputData, op: BinaryOp) -> Instruction {
    match op {
        BinaryOp::EqEq => Instruction::eq(left, right, out),
        BinaryOp::NotEq => Instruction::ne(left, right, out),
        BinaryOp::EqEqEq => Instruction::strict_eq(left, right, out),
        BinaryOp::NotEqEq => Instruction::strict_ne(left, right, out),
        BinaryOp::Lt => Instruction::lt(left, right, out),
        BinaryOp::LtEq => Instruction::lt_eq(left, right, out),
        BinaryOp::Gt => Instruction::gt(left, right, out),
        BinaryOp::GtEq => Instruction::gt_eq(left, right, out),
        BinaryOp::LShift => Instruction::l_shift(left, right, out),
        BinaryOp::RShift => Instruction::r_shift(left, right, out),
        BinaryOp::ZeroFillRShift => Instruction::zero_fill_r_shift(left, right, out),
        BinaryOp::Add => Instruction::add(left, right, out),
        BinaryOp::Sub => Instruction::sub(left, right, out),
        BinaryOp::Mul => Instruction::mul(left, right, out),
        BinaryOp::Div => Instruction::div(left, right, out),
        BinaryOp::Mod => Instruction::mod_(left, right, out),
        BinaryOp::BitOr => Instruction::b_or(left, right, out),
        BinaryOp::BitXor => Instruction::b_xor(left, right, out),
        BinaryOp::BitAnd => Instruction::b_and(left, right, out),
        BinaryOp::LogicalOr => Instruction::l_or(left, right, out),
        BinaryOp::LogicalAnd => Instruction::l_and(left, right, out),
        BinaryOp::In => Instruction::in_(left, right, out),
        BinaryOp::InstanceOf => Instruction::instance_of(left, right, out),
        BinaryOp::Exp => Instruction::exp(left, right, out),
        BinaryOp::NullishCoalescing => Instruction::nullish_coalescing(left, right, out),
    }
}
