use crate::{ByteCodegen, Res};
use anyhow::anyhow;
use yavashark_bytecode::Instruction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LabelName {
    Loop,
    Label(String),
}

impl ByteCodegen {
    pub fn backpatch(&mut self, name: String, target: usize) {
        self.label_backpatch.push((LabelName::Label(name), target));
    }

    pub fn backpatch_break(&mut self, target: usize) {
        self.label_backpatch.push((LabelName::Loop, target));
    }

    pub fn compile_label_jump(&mut self, name: &str) -> Res {
        let target = self
            .labels
            .iter()
            .rev()
            .find(|(n, _)| n == name)
            .ok_or(anyhow!("Label {} not found", name))?
            .1;

        self.instructions.push(Instruction::Jmp(target));

        Ok(())
    }
}
