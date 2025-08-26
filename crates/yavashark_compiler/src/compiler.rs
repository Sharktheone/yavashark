mod function;
mod pat;
mod statement;

use crate::Res;
use swc_ecma_ast::{Pat, Stmt};
use yavashark_bytecode::ConstValue;
use yavashark_bytecode::control::ControlBlock;
use yavashark_bytecode::data::{Acc, Label, Stack};
use yavashark_bytecode::instructions::Instruction;

#[derive(Debug, Clone, Default)]
pub struct Compiler {
    pub instructions: Vec<Instruction>,
    pub variables: Vec<String>,
    pub labeled: Vec<String>,
    pub active_labeled: Vec<Label>,
    pub literals: Vec<ConstValue>,
    pub control: Vec<ControlBlock>,
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

    pub fn compile_params<'a>(params: impl Iterator<Item = &'a Pat>) -> Res<(Self, Vec<u32>)> {
        let mut this = Self::new();

        let (low, high) = params.size_hint();
        let num_params = high.unwrap_or(low);
        let mut param_defs = Vec::with_capacity(num_params);

        for pat in params {
            this.compile_pat(pat, Acc)?;
            param_defs.push(this.instructions.len() as u32);
            this.reset_allocs();
        }

        Ok((this, param_defs))
    }

    pub fn reset_allocs(&mut self) {
        self.labeled.clear();
        self.active_labeled.clear();
        self.labels.clear();
        self.loop_label = None;
        self.label_backpatch.clear();
        self.used_registers.clear();
        self.stack_ptr = 0;
        self.max_stack_size = 0;
        self.stack_to_deallloc.clear();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LabelName {
    Loop,
    Label(String),
}
