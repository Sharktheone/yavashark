use crate::VM;
use yavashark_bytecode::VarName;
use yavashark_env::{Error, Value};
use yavashark_string::YSString;

pub trait ValueExt {
    fn get_member(&self, member: VarName, vm: &mut impl VM) -> Result<Self, Error>
    where
        Self: Sized;
}

impl ValueExt for Value {
    fn get_member(&self, member: VarName, vm: &mut impl VM) -> Result<Self, Error> {
        let member = Self::String(YSString::from_ref(
            vm.var_name(member)
                .ok_or(Error::reference("member name not found"))?,
        ));

        self.get_property(&member, vm.get_realm())
    }
}
