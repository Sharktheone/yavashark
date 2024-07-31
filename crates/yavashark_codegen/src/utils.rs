use crate::ByteCodegen;
use yavashark_bytecode::VarName;

impl ByteCodegen {
    pub fn allocate_variable(&mut self, name: String) -> VarName {
        let idx = self.variables.len();
        self.variables.push(name);
        idx as VarName
    }
}
