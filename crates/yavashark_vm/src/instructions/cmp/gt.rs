

use yavashark_bytecode::{Reg, VarName};

use crate::VM;

pub fn gt(lhs: VarName, rhs: VarName, vm: &mut VM) {
    let lhs = vm.get_variable(lhs);
    let rhs = vm.get_variable(rhs);

    vm.set_acc((lhs > rhs).into());
}

pub fn gt_acc(reg: Reg, vm: &mut VM) {
    let rhs = vm.get_register(reg);
    let lhs = vm.acc();

    vm.set_acc((lhs > rhs).into());
}

pub fn gt_reg(rhs: Reg, lhs: Reg, vm: &mut VM) {
    let rhs = vm.get_register(rhs);
    let lhs = vm.get_register(lhs);

    vm.set_acc((lhs > rhs).into());
}


