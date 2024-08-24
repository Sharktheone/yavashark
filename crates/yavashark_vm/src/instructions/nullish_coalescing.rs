use crate::VM;
use yavashark_bytecode::{Reg, VarName};

pub fn nullish_coalescing(lhs: VarName, rhs: VarName, vm: &mut VM) {
    let lhs = vm.get_variable(lhs);
    let rhs = vm.get_variable(rhs);

    if lhs.is_nullish() {
        vm.set_acc(rhs);
    } else {
        vm.set_acc(lhs);
    }
}

pub fn nullish_coalescing_acc(reg: Reg, vm: &mut VM) {
    let acc = vm.acc();
    let reg = vm.get_register(reg);

    if acc.is_nullish() {
        vm.set_acc(reg);
    } else {
        vm.set_acc(acc);
    }
}

pub fn nullish_coalescing_reg(reg1: Reg, reg2: Reg, vm: &mut VM) {
    let reg1 = vm.get_register(reg1);
    let reg2 = vm.get_register(reg2);

    if reg1.is_nullish() {
        vm.set_acc(reg2);
    } else {
        vm.set_acc(reg1);
    }
}
