use crate::Compiler;
use std::borrow::Cow;
use yavashark_bytecode::ConstValue;
use yavashark_bytecode::control::{ControlBlock, TryBlock};
use yavashark_bytecode::data::{ConstIdx, ControlIdx, DataType, Label, OutputDataType, Reg, Stack, VarName};

impl Compiler {
    pub fn alloc_var<'a>(&mut self, name: impl Into<Cow<'a, str>>) -> VarName {
        let name = name.into();

        if let Some(var) = self
            .variables
            .iter()
            .rposition(|x| x.as_str() == name.as_ref())
        {
            return VarName(var as u32);
        }

        let var = self.variables.len();
        self.variables.push(name.into_owned());

        VarName(var as u32)
    }

    pub fn alloc_const(&mut self, val: impl Into<ConstValue>) -> ConstIdx {
        let val = val.into();

        if let Some(c_idx) = self.literals.iter().position(|x| *x == val) {
            return ConstIdx(c_idx as u32);
        }

        let c_idx = self.literals.len();
        self.literals.push(val);

        ConstIdx(c_idx as u32)
    }

    pub fn alloc_label<'a>(&mut self, label: impl Into<Cow<'a, str>>) -> Label {
        let label = label.into();

        if let Some(l_idx) = self.labeled.iter().position(|x| *x == label) {
            return Label(l_idx as u32);
        }

        let l_idx = self.labeled.len();
        self.labeled.push(label.into_owned());

        let lbl = Label(l_idx as u32);

        self.active_labeled.push(lbl);

        lbl
    }

    pub fn dealloc_label(&mut self) {
        self.active_labeled.pop();
    }

    #[must_use]
    pub fn has_label(&self, label: &str) -> bool {
        self.labeled.iter().any(|x| x == label)
    }

    #[must_use]
    pub fn get_label(&self, label: &str) -> Option<Label> {
        self.labeled
            .iter()
            .position(|x| x == label)
            .map(|x| Label(x as u32))
    }

    pub fn alloc_reg(&mut self) -> Option<Reg> {
        self.used_registers
            .iter_mut()
            .position(|x| {
                if *x {
                    false
                } else {
                    *x = true;
                    true
                }
            })
            .map(|x| Reg(x as u8))
    }

    pub fn dealloc_reg(&mut self, reg: Reg) {
        if let Some(reg) = self.used_registers.get_mut(reg.0 as usize) {
            *reg = false;
        }
    }

    pub fn alloc_stack(&mut self) -> Stack {
        let stack = Stack(self.stack_ptr);
        self.stack_ptr += 1;
        stack
    }

    pub fn delloc_stack(&mut self, stack: Stack) {
        if stack.0 == self.stack_ptr - 1 {
            self.stack_ptr -= 1;
            return;
        }

        self.stack_to_deallloc.push(stack);
    }

    pub fn alloc_reg_or_stack(&mut self) -> OutputDataType {
        self.alloc_reg()
            .map(OutputDataType::Reg)
            .unwrap_or_else(|| OutputDataType::Stack(self.alloc_stack()))
    }

    pub fn dealloc(&mut self, data: impl Into<DataType>) {
        match data.into() {
            DataType::Reg(reg) => self.dealloc_reg(reg),
            DataType::Stack(stack) => self.delloc_stack(stack),
            _ => {}
        }
    }
    
    pub fn new_try_block(&mut self) -> ControlIdx {
        let idx = self.control.len();
        self.control.push(ControlBlock::Try(Default::default()));
        ControlIdx(idx as u32)
        
    }
    
    pub fn set_try(&mut self, idx: ControlIdx, block: TryBlock) {
        if let Some(control) = self.control.get_mut(idx.0 as usize) {
            *control = ControlBlock::Try(block);
        }
    }
}
