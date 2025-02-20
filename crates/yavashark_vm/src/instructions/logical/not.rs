use crate::VM;
use yavashark_bytecode::VarName;
use yavashark_env::{Res, Value};

pub fn logical_not(name: VarName, vm: &mut impl VM) -> Res {
    let value = vm.get_variable(name)?;

    vm.set_acc(Value::Boolean(!value.is_truthy()));

    Ok(())
}

pub fn logical_not_acc(vm: &mut impl VM) -> Res {
    vm.set_acc(Value::Boolean(!vm.acc().is_truthy()));

    Ok(())
}
