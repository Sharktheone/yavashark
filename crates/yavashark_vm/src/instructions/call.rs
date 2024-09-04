use crate::value_ext::ValueExt;
use crate::VM;
use yavashark_bytecode::{Reg, VarName};
use yavashark_env::ControlResult;

pub fn call(num_args: u16, var_name: VarName, vm: &mut VM) -> ControlResult {
    let func = vm.get_variable(var_name);

    let this = vm.get_this();

    let args = vm.get_args(num_args);

    func.call(vm.get_context(), args, this)
}

pub fn call_reg(num_args: u16, reg: Reg, vm: &mut VM) -> ControlResult {
    let func = vm.get_register(reg);

    let this = vm.get_this();

    let args = vm.get_args(num_args);

    func.call(vm.get_context(), args, this)
}

pub fn call_acc(num_args: u16, vm: &mut VM) -> ControlResult {
    let func = vm.acc();

    let this = vm.get_this();

    let args = vm.get_args(num_args);

    func.call(vm.get_context(), args, this)
}

pub fn call_member(num_args: u16, target: VarName, member: VarName, vm: &mut VM) -> ControlResult {
    let value = vm.get_variable(target);
    let func = value.get_member(member, vm)?;

    let this = vm.get_this();

    let args = vm.get_args(num_args);

    func.call(vm.get_context(), args, this)
}

pub fn call_member_reg(num_args: u16, target: Reg, member: VarName, vm: &mut VM) -> ControlResult {
    let value = vm.get_register(target);
    let func = value.get_member(member, vm)?;

    let this = vm.get_this();

    let args = vm.get_args(num_args);

    func.call(vm.get_context(), args, this)
}

pub fn call_member_acc(num_args: u16, member: VarName, vm: &mut VM) -> ControlResult {
    let value = vm.acc();
    let func = value.get_member(member, vm)?;

    let this = vm.get_this();

    let args = vm.get_args(num_args);

    func.call(vm.get_context(), args, this)
}
