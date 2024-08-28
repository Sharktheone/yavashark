use crate::VM;
use yavashark_bytecode::Reg;

pub fn this_acc(vm: &mut VM) {
    let this = vm.get_this();
    vm.set_acc(this);
}

pub fn this_reg(reg: Reg, vm: &mut VM) {
    let this = vm.get_this();

    vm.set_register(reg, this);
}
