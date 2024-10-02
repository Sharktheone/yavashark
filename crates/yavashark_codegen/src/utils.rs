use crate::ByteCodegen;
use yavashark_bytecode::VarName;

impl ByteCodegen {
    pub fn allocate_variable(&mut self, name: String) -> VarName {
        
        
        if let Some(idx) = self.variables.iter().position(|n| n == &name) {
            return idx as VarName;
        }
        
        
        let idx = self.variables.len();
        self.variables.push(name);
        idx as VarName
    }

    pub fn allocate_literal(&mut self, val: yavashark_bytecode::ConstValue) -> VarName {
        if let Some(idx) = self.literals.iter().position(|n| n == &val) {
            return idx as VarName;
        }
        
        
        let idx = self.literals.len();
        self.literals.push(val);
        idx as VarName
    }
}
