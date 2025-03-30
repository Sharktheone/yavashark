use yavashark_bytecode::jmp::Test;
use crate::{Compiler, Res};


impl Compiler {
    pub fn compile_if(&mut self, i: &swc_ecma_ast::IfStmt) -> Res {
        let cond = self.compile_test_expr(&i.test)?;
        
        if cond == Test::Unconditional {
            //only compile else
        } else if cond == Test::Never {
            //only compile if
        } else {
            //compile if and else plus jump
        }
        
        
        Ok(())
    }
    
}