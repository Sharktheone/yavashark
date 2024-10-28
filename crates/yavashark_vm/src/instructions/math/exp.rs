#![allow(clippy::similar_names)]

use crate::VM;
use yavashark_bytecode::{Reg, VarName};
use yavashark_env::Res;

pub fn exp(lhs: VarName, rhs: VarName, vm: &mut VM) -> Res {
    let lhs = vm.get_variable(lhs)?;
    let rhs = vm.get_variable(rhs)?;

    let realm = vm.get_realm();

    let res = lhs.pow(&rhs, realm)?;

    vm.set_acc(res);

    Ok(())
}

pub fn exp_acc(reg: Reg, vm: &mut VM) -> Res {
    let acc = vm.acc();
    let reg = vm.get_register(reg)?;

    let realm = vm.get_realm();

    let res = acc.pow(&reg, realm)?;

    vm.set_acc(res);

    Ok(())
}

pub fn exp_reg(reg1: Reg, reg2: Reg, vm: &mut VM) -> Res {
    let reg1 = vm.get_register(reg1)?;
    let reg2 = vm.get_register(reg2)?;

    let realm = vm.get_realm();

    let res = reg1.pow(&reg2, realm)?;

    vm.set_acc(res);

    Ok(())
}
