use crate::ByteCodegen;
use yavashark_bytecode::VarName;

impl ByteCodegen {
    pub fn allocate_variable(&mut self, name: String) -> VarName {
        let idx = self.variables.len();
        self.variables.push(name);
        idx as VarName
    }
    
    pub fn allocate_literal(&mut self, val: yavashark_bytecode::ConstValue) -> VarName {
        let idx = self.literals.len();
        self.literals.push(val);
        idx as VarName
    }
}
