use yavashark_env::Res;
use crate::data::{Data, OutputData};
use crate::VM;

pub fn l_not(data: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let data = data.get(vm)?;
    let result = (!data.is_truthy()).into();

    output.set(result, vm)
}

pub fn l_or(left: impl Data, right: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;
    let result = left.or(&right, vm.get_realm())?;

    output.set(result, vm)
}

pub fn l_and(left: impl Data, right: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;
    let result = left.and(&right, vm.get_realm())?;

    output.set(result, vm)
}