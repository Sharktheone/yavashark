use crate::VM;
use yavashark_bytecode::{Reg, VarName};
use yavashark_env::Res;

pub fn inc(name: VarName, vm: &mut impl VM) -> Res {
    let val = vm.get_variable(name)?;

    let result = val + 1.into();

    vm.set_acc(result);

    Ok(())
}

pub fn inc_acc(vm: &mut impl VM) -> Res {
    let acc = vm.acc();

    let result = acc + 1.into();

    vm.set_acc(result);

    Ok(())
}

pub fn inc_reg(name: Reg, vm: &mut impl VM) -> Res {
    let val = vm.get_register(name)?;

    let result = val + 1.into();

    vm.set_acc(result);

    Ok(())
}
