use swc_ecma_ast::Ident;
use yavashark_bytecode::Instruction;
use crate::{ByteCodegen, Res};

impl ByteCodegen {
    pub fn compile_ident(&mut self, stmt: &Ident) -> Res {
        
        let var_name = self.allocate_variable(stmt.sym.as_str().to_string());
        
        self.instructions.push(Instruction::LoadEnv(var_name));
        
        Ok(())
    }
}
