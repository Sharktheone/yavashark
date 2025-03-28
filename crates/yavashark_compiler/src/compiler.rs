mod statement;

use yavashark_bytecode::instructions::Instruction;
use yavashark_bytecode::ConstValue;

pub struct Compiler {
    pub instructions: Vec<Instruction>,
    pub variables: Vec<String>,
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
