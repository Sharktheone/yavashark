use crate::{Compiler, Res};
use yavashark_bytecode::instructions::Instruction;

impl Compiler {
    pub fn compile_return(&mut self, ret: &swc_ecma_ast::ReturnStmt) -> Res {
        if let Some(ret) = &ret.arg {
            let val = self.compile_expr_data_acc(&ret)?;

            self.instructions.push(Instruction::return_value(val));
        } else {
            self.instructions.push(Instruction::Return);
        }

        Ok(())
    }
}
