use crate::VM;
use yavashark_bytecode::{Reg, VarName};
use yavashark_env::Res;

pub fn zero_fill_rshift(lhs: VarName, rhs: VarName, vm: &mut impl VM) -> Res {
    let lhs = vm.get_variable(lhs)?;
    let rhs = vm.get_variable(rhs)?;

    let result = rhs.ushr(&lhs, vm.get_realm())?;

    vm.set_acc(result);

    Ok(())
}

pub fn zero_fill_rshift_acc(reg: Reg, vm: &mut impl VM) -> Res {
    let rhs = vm.get_register(reg)?;
    let lhs = vm.acc();

    let result = rhs.ushr(&lhs, vm.get_realm())?;

    vm.set_acc(result);

    Ok(())
}

pub fn zero_fill_rshift_reg(rhs: Reg, lhs: Reg, vm: &mut impl VM) -> Res {
    let rhs = vm.get_register(rhs)?;
    let lhs = vm.get_register(lhs)?;

    let result = rhs.ushr(&lhs, vm.get_realm())?;

    vm.set_acc(result);

    Ok(())
}
