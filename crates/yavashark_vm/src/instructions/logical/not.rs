use crate::VM;
use yavashark_bytecode::VarName;
use yavashark_env::Value;

pub fn logical_not(name: VarName, vm: &mut VM) {
    let value = vm.get_variable(name);

    vm.set_acc(Value::Boolean(!value.is_truthy()));
}

pub fn logical_not_acc(vm: &mut VM) {
    vm.set_acc(Value::Boolean(!vm.acc().is_truthy()));
}
