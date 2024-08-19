use crate::VM;
use yavashark_bytecode::{Reg, VarName};

pub fn modulo(lhs: VarName, rhs: VarName, vm: &mut VM) {
    let lhs = vm.get_variable(lhs);
    let rhs = vm.get_variable(rhs);

    let result = lhs % rhs;

    vm.set_acc(result);
}

pub fn mod_acc_reg(reg: Reg, vm: &mut VM) {
    let acc = vm.acc();
    let reg = vm.get_register(reg);

    let result = acc % reg;

    vm.set_acc(result);
}

pub fn mod_reg(reg1: Reg, reg2: Reg, vm: &mut VM) {
    let reg1 = vm.get_register(reg1);
    let reg2 = vm.get_register(reg2);

    let result = reg1 % reg2;

    vm.set_acc(result);
}
