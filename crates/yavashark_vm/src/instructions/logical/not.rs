use crate::VM;
use yavashark_bytecode::VarName;

pub fn logical_not(name: VarName, vm: &mut VM) {
    let value = vm.get_variable(name);

    vm.set_acc(!value);
}

pub fn logical_not_acc(vm: &mut VM) {
    vm.set_acc(!vm.acc());
}
