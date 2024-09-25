use crate::{ByteCodegen, LabelName, Res};
use log::debug;
use std::fmt::format;
use swc_ecma_ast::{ExprStmt, LabeledStmt, ReturnStmt};
use yavashark_bytecode::Instruction;

impl ByteCodegen {
    pub fn compile_labeled(&mut self, stmt: &LabeledStmt) -> Res {
        let name = stmt.label.sym.to_string();

        self.labels.push((name.clone(), self.instructions.len()));

        self.compile_statement(&stmt.body)?;

        let lbl = self.labels.pop();

        
        self.backpatch_label(name, self.instructions.len());

        Ok(())
    }

    fn backpatch_label(&mut self, name: String, pos: usize) {
        let name = LabelName::Label(name);
        
        let (backpatch, rem) = self
            .label_backpatch
            .drain(..)
            .partition(|(n, _)| *n == name);

        self.label_backpatch = rem;

        for (_, p) in backpatch {
            self.instructions[p] = Instruction::Jmp(pos);
        }
    }
}
