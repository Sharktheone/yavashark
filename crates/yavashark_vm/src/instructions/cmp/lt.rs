use crate::VM;
use yavashark_bytecode::{Reg, VarName};
use yavashark_env::Res;

pub fn lt(lhs: VarName, rhs: VarName, vm: &mut impl VM) -> Res {
    let lhs = vm.get_variable(lhs)?;
    let rhs = vm.get_variable(rhs)?;

    vm.set_acc((lhs < rhs).into());

    Ok(())
}

pub fn lt_acc(reg: Reg, vm: &mut impl VM) -> Res {
    let rhs = vm.get_register(reg)?;
    let lhs = vm.acc();

    vm.set_acc((lhs < rhs).into());

    Ok(())
}

pub fn lt_reg(rhs: Reg, lhs: Reg, vm: &mut impl VM) -> Res {
    let rhs = vm.get_register(rhs)?;
    let lhs = vm.get_register(lhs)?;

    vm.set_acc((lhs < rhs).into());

    Ok(())
}
