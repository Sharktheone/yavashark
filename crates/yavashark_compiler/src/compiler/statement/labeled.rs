use crate::{Compiler, Res};
use swc_ecma_ast::LabeledStmt;
use yavashark_bytecode::instructions::Instruction;

impl Compiler {
    pub fn compile_labeled(&mut self, labeled: &LabeledStmt) -> Res {
        self.alloc_label(labeled.label.sym.to_string());

        self.instructions.push(Instruction::PushScope);
        self.compile_stmt(&labeled.body)?;
        self.instructions.push(Instruction::PopScope);

        self.dealloc_label();

        Ok(())
    }
}
