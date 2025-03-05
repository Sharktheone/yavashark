use crate::VM;
use yavashark_bytecode::{Reg, VarName};
use yavashark_env::Res;

pub fn sub(lhs: VarName, rhs: VarName, vm: &mut impl VM) -> Res {
    let lhs = vm.get_variable(lhs)?;
    let rhs = vm.get_variable(rhs)?;

    let result = lhs.sub(&rhs, vm.get_realm())?;

    vm.set_acc(result);
    Ok(())
}

pub fn sub_acc_reg(reg: Reg, vm: &mut impl VM) -> Res {
    let acc = vm.acc();
    let reg = vm.get_register(reg)?;

    let result = acc.sub(&reg, vm.get_realm())?;

    vm.set_acc(result);
    Ok(())
}

pub fn sub_reg(reg1: Reg, reg2: Reg, vm: &mut impl VM) -> Res {
    let reg1 = vm.get_register(reg1)?;
    let reg2 = vm.get_register(reg2)?;

    let result = reg1.sub(&reg2, vm.get_realm())?;

    vm.set_acc(result);
    Ok(())
}
