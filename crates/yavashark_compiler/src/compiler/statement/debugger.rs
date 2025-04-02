use crate::{Compiler, Res};
use yavashark_bytecode::instructions::Instruction;

impl Compiler {
    pub fn compile_debugger(&mut self, _: &swc_ecma_ast::DebuggerStmt) -> Res {
        self.instructions.push(Instruction::Debugger);

        Ok(())
    }
}
