use crate::ByteCodegen;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LabelName {
    LoopBreak,
    LoopContinue,
    Label(String),
}



impl ByteCodegen {
    pub fn backpatch(&mut self, name: String, target: usize) {
        self.label_backpatch.push((LabelName::Label(name), target));
    }

    pub fn backpatch_break(&mut self, target: usize) {
        self.label_backpatch.push((LabelName::LoopBreak, target));
    }

    pub fn backpatch_continue(&mut self, target: usize) {
        self.label_backpatch.push((LabelName::LoopContinue, target));
    }
}