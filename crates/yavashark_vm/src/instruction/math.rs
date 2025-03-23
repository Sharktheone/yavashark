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