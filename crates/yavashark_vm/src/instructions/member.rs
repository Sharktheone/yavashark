use crate::value_ext::ValueExt;
use crate::VM;
use yavashark_bytecode::{Reg, VarName};
use yavashark_env::ControlResult;

pub fn load_member(target: VarName, member: VarName, vm: &mut impl VM) -> ControlResult {
    let value = vm.get_variable(target)?;
    let member = value.get_member(member, vm)?;
    vm.set_acc(member);

    Ok(())
}

pub fn load_member_acc(member: VarName, vm: &mut impl VM) -> ControlResult {
    let acc = vm.acc();
    let value = acc.get_member(member, vm)?;
    vm.set_acc(value);

    Ok(())
}

pub fn load_member_reg(target: Reg, member: VarName, vm: &mut impl VM) -> ControlResult {
    let value = vm.get_register(target)?;
    let member = value.get_member(member, vm)?;
    vm.set_acc(member);

    Ok(())
}
