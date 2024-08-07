use crate::{ByteCodegen, Res};
use swc_ecma_ast::{BinaryOp, BinExpr, ThisExpr};
use yavashark_bytecode::Instruction;

impl ByteCodegen {
    pub fn compile_bin(&mut self, stmt: &BinExpr) -> Res {
        self.compile_expr(&stmt.right, stmt.span)?;
        self.instructions.push(Instruction::PushAcc):
        
        self.compile_expr(&stmt.left, stmt.span)?;
        
        self.instructions.push(Instruction::PopToReg(0));
        
        
        
        
        match stmt.op {
            BinaryOp::Add => self.instructions.push(Instruction::AddAccReg(0)),
            BinaryOp::Sub => self.instructions.push(Instruction::SubAccReg(0)),
            BinaryOp::Mul => self.instructions.push(Instruction::MulAccReg(0)),
            BinaryOp::Div => self.instructions.push(Instruction::DivAccReg(0)),
            BinaryOp::Mod => todo!(),
            BinaryOp::BitOr => self.instructions.push(Instruction::LOrAcc(0)),
            BinaryOp::BitXor => self.instructions.push(Instruction::LXorAcc(0)),
            BinaryOp::BitAnd => self.instructions.push(Instruction::LAndAcc(0)),
            
            o => todo!("{:?}", o),
        }
        
        
        Ok(())
    }
}
