mod statement;

use crate::Res;
use swc_ecma_ast::Stmt;
use yavashark_bytecode::ConstValue;
use yavashark_bytecode::data::{Label, Stack};
use yavashark_bytecode::instructions::Instruction;

#[derive(Debug, Clone, Default)]
pub struct Compiler {
    pub instructions: Vec<Instruction>,
    pub variables: Vec<String>,
    pub labeled: Vec<String>,
    pub active_labeled: Vec<Label>,
    pub literals: Vec<ConstValue>,
    labels: Vec<(String, usize)>,
    loop_label: Option<usize>,
    label_backpatch: Vec<(LabelName, usize)>,
    pub used_registers: Vec<bool>,
    pub stack_ptr: u32,
    pub max_stack_size: u32,
    pub stack_to_deallloc: Vec<Stack>,
}

impl Compiler {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn compile(stmt: &[Stmt]) -> Res<Self> {
        let mut this = Self::new();

        this.compile_stmts(stmt)?;

        Ok(this)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LabelName {
    Loop,
    Label(String),
}
