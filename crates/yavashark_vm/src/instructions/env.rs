use crate::VM;
use yavashark_bytecode::{Reg, VarName};
use yavashark_env::Res;

pub fn load_env(name: VarName, vm: &mut impl VM) -> Res {
    let value = vm.get_variable(name)?;

    vm.set_acc(value);

    Ok(())
}

pub fn load_env_reg(name: VarName, reg: Reg, vm: &mut impl VM) -> Res {
    let value = vm.get_variable(name)?;

    vm.set_register(reg, value)?;

    Ok(())
}
