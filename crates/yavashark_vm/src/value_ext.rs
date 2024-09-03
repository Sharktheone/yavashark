use yavashark_bytecode::VarName;
use yavashark_env::{Error, Value};
use crate::VM;

pub trait ValueExt {
    fn get_member(&self, member: VarName, vm: &mut VM) -> Result<Self, Error>;
}


impl ValueExt for Value {
    fn get_member(&self, member: VarName, vm: &mut VM) -> Result<Self, Error> {
        let member = Value::String(vm.var_name(member).to_string());
        
        self.get_property(&member, vm.get_context())
    }
}