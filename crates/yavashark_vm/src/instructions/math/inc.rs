use crate::VM;
use yavashark_bytecode::{Reg, VarName};

pub fn inc(name: VarName, vm: &mut VM) {
    let val = vm.get_variable(name);

    let result = val + 1.into();

    vm.set_acc(result);
}

pub fn inc_acc(vm: &mut VM) {
    let acc = vm.acc();

    let result = acc + 1.into();

    vm.set_acc(result);
}

pub fn inc_reg(name: Reg, vm: &mut VM) {
    let val = vm.get_register(name);

    let result = val + 1.into();

    vm.set_acc(result);
}
