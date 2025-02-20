use crate::VM;
use yavashark_bytecode::{Reg, VarName};
use yavashark_env::Res;

pub fn logical_or(lhs: VarName, rhs: VarName, vm: &mut impl VM) -> Res {
    let lhs = vm.get_variable(lhs)?;
    let rhs = vm.get_variable(rhs)?;

    vm.set_acc(lhs.log_or(rhs));

    Ok(())
}

pub fn logical_or_acc(reg: Reg, vm: &mut impl VM) -> Res {
    let rhs = vm.get_register(reg)?;
    let lhs = vm.acc();

    vm.set_acc(lhs.log_or(rhs));

    Ok(())
}
