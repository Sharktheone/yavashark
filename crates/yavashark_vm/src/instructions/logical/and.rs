use crate::VM;
use yavashark_bytecode::{Reg, VarName};

pub fn logical_and(lhs: VarName, rhs: VarName, vm: &mut VM) {
    let lhs = vm.get_variable(lhs);
    let rhs = vm.get_variable(rhs);

    vm.set_acc(lhs.log_and(rhs));
}

pub fn logical_and_acc(reg: Reg, vm: &mut VM) {
    let rhs = vm.get_register(reg);
    let lhs = vm.acc();

    vm.set_acc(lhs.log_and(rhs));
}