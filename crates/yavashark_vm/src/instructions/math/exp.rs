use yavashark_bytecode::{Reg, VarName};
use crate::VM;

pub fn exp(lhs: VarName, rhs: VarName, vm: &mut VM) {
    let lhs = vm.get_variable(lhs);
    let rhs = vm.get_variable(rhs);

    vm.set_acc(lhs.pow(&rhs));
}


pub fn exp_acc(reg: Reg, vm: &mut VM) {
    let acc = vm.acc();
    let reg = vm.get_register(reg);

    vm.set_acc(acc.pow(&reg));
}

pub fn exp_reg(reg1: Reg, reg2: Reg, vm: &mut VM) {
    let reg1 = vm.get_register(reg1);
    let reg2 = vm.get_register(reg2);

    vm.set_acc(reg1.pow(&reg2));
}