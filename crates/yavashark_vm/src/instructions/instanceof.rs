
use yavashark_bytecode::{Reg, VarName};
use yavashark_env::ControlResult;
use crate::VM;

pub fn instance_of(lhs: VarName, rhs: VarName, vm: &mut VM) -> ControlResult {
    let lhs = vm.get_variable(lhs);
    let rhs = vm.get_variable(rhs);

    vm.set_acc(lhs.instance_of(&rhs, vm.get_context()).into());

    Ok(())
}


pub fn instance_of_acc(reg: Reg, vm: &mut VM) -> ControlResult {
    let rhs = vm.get_register(reg);
    let lhs = vm.acc();

    vm.set_acc(lhs.instance_of(&rhs, vm.get_context()).into());

    Ok(())
}


pub fn instance_of_reg(rhs: Reg, lhs: Reg, vm: &mut VM) -> ControlResult {
    let rhs = vm.get_register(rhs);
    let lhs = vm.get_register(lhs);

    vm.set_acc(lhs.instance_of(&rhs, vm.get_context()).into());

    Ok(())
}
