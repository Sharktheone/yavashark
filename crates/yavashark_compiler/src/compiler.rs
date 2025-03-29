mod statement;

use yavashark_bytecode::instructions::Instruction;
use yavashark_bytecode::ConstValue;
use yavashark_bytecode::data::Label;

pub struct Compiler {
    pub instructions: Vec<Instruction>,
    pub variables: Vec<String>,
    pub labeled: Vec<String>,
    pub active_labeled: Vec<Label>,
    pub literals: Vec<ConstValue>,
    labels: Vec<(String, usize)>,
    loop_label: Option<usize>,
    label_backpatch: Vec<(LabelName, usize)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LabelName {
    Loop,
    Label(String),
}
