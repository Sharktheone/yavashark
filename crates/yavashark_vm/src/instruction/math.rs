use yavashark_env::Res;
use crate::data::{Data, OutputData};
use crate::VM;

pub fn add(left: impl Data, right: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;
    let result = left.add(&right, vm.get_realm())?;

    output.set(result, vm)
}

pub fn sub(left: impl Data, right: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;
    let result = left.sub(&right, vm.get_realm())?;

    output.set(result, vm)
}

pub fn mul(left: impl Data, right: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;
    let result = left.mul(&right, vm.get_realm())?;

    output.set(result, vm)
}

pub fn div(left: impl Data, right: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;
    let result = left.div(&right, vm.get_realm())?;

    output.set(result, vm)
}

pub fn mod_(left: impl Data, right: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;
    let result = left.rem(&right, vm.get_realm())?;

    output.set(result, vm)
}

pub fn exp(left: impl Data, right: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;
    let result = left.exp(&right, vm.get_realm())?;

    output.set(result, vm)
}

pub fn dec(data: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let data = data.get(vm)?;
    let result = data.sub(&1.into(), vm.get_realm())?;

    output.set(result, vm)
}

pub fn inc(data: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let data = data.get(vm)?;
    let result = data.add(&1.into(), vm.get_realm())?;

    output.set(result, vm)
}