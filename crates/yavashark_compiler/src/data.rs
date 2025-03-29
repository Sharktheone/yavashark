use std::borrow::Cow;
use yavashark_bytecode::ConstValue;
use yavashark_bytecode::data::{ConstIdx, Label, VarName};
use crate::Compiler;

impl Compiler {
    pub fn alloc_var<'a>(&mut self, name: impl Into<Cow<'a, str>>) -> VarName {
        let name = name.into();
        
        if let Some(var) =  self.variables.iter().rposition(|x| x.as_str() == name.as_ref()) {
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

    pub fn has_label(&self, label: &str) -> bool {
        self.labeled.iter().any(|x| x == label)
    }
    
    pub fn get_label(&self, label: &str) -> Option<Label> {
        self.labeled.iter().position(|x| x == label).map(|x| Label(x as u32))
    }
}