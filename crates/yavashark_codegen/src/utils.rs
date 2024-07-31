use yavashark_bytecode::VarName;
use crate::ByteCodegen;

impl ByteCodegen {
    pub fn allocate_variable(&mut self, name: String) -> VarName {
        let idx = self.variables.len();
        self.variables.push(name);
        idx as VarName
    }
}