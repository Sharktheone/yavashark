use crate::VM;
use yavashark_bytecode::{Reg, VarName};
use yavashark_env::ControlResult;

pub fn in_(lhs: VarName, rhs: VarName, vm: &mut VM) -> ControlResult {
    let lhs = vm.get_variable(lhs)?;
    let rhs = vm.get_variable(rhs)?;

    vm.set_acc(rhs.contains_key(&lhs)?.into());

    Ok(())
}

pub fn in_acc(reg: Reg, vm: &mut VM) -> ControlResult {
    let rhs = vm.get_register(reg)?;
    let lhs = vm.acc();

    vm.set_acc(rhs.contains_key(&lhs)?.into());

    Ok(())
}

pub fn in_reg(rhs: Reg, lhs: Reg, vm: &mut VM) -> ControlResult {
    let rhs = vm.get_register(rhs)?;
    let lhs = vm.get_register(lhs)?;

    vm.set_acc(rhs.contains_key(&lhs)?.into());

    Ok(())
}
