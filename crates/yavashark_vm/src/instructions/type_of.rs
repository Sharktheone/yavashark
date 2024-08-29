use yavashark_bytecode::VarName;
use crate::VM;

pub fn type_of(var: VarName, vm: &mut  VM) {
    let value = vm.get_variable(var);

    vm.set_acc(value.type_of().into());
}


pub fn type_of_acc(vm: &mut VM) {
    let value = vm.acc();

    vm.set_acc(value.type_of().into());
}
