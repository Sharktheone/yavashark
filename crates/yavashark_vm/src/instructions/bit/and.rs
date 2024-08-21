use yavashark_bytecode::{Reg, VarName};
use crate::VM;

pub fn bitwise_and(lhs: VarName, rhs: VarName, vm: &mut VM) {
    let lhs = vm.get_variable(lhs);
    let rhs = vm.get_variable(rhs);

    vm.set_acc(lhs & rhs);
}

pub fn bitwise_and_acc(reg: Reg, vm: &mut  VM) {
    let rhs = vm.get_register(reg);
    let lhs = vm.acc();

    vm.set_acc(lhs & rhs);
}


pub fn bitwise_and_reg(rhs: Reg, lhs: Reg, vm: &mut  VM) {
    let rhs = vm.get_register(rhs);
    let lhs = vm.get_register(lhs);

    vm.set_acc(lhs & rhs);
}
