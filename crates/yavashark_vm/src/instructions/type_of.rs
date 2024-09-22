use crate::VM;
use yavashark_bytecode::VarName;
use yavashark_env::Res;

pub fn type_of(var: VarName, vm: &mut VM) -> Res {
    let value = vm.get_variable(var)?;

    vm.set_acc(value.type_of().into());

    Ok(())
}

pub fn type_of_acc(vm: &mut VM) -> Res {
    let value = vm.acc();

    vm.set_acc(value.type_of().into());

    Ok(())
}
