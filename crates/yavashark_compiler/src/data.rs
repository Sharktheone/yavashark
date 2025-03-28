use std::borrow::Cow;
use yavashark_bytecode::ConstValue;
use yavashark_bytecode::data::{ConstIdx, VarName};
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
}