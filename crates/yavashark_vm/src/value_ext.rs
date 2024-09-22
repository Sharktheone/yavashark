use crate::VM;
use yavashark_bytecode::VarName;
use yavashark_env::{Error, Value};

pub trait ValueExt {
    fn get_member(&self, member: VarName, vm: &mut VM) -> Result<Self, Error> where Self: Sized;
}

impl ValueExt for Value {
    fn get_member(&self, member: VarName, vm: &mut VM) -> Result<Self, Error> {
        let member = Value::String(vm.var_name(member).ok_or(Error::reference("member name not found"))?.to_string());

        self.get_property(&member, vm.get_context())
    }
}
