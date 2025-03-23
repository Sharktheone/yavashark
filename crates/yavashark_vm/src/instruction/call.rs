use yavashark_env::Res;
use crate::data::{Data, OutputData};
use crate::VM;

pub fn call(func: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let func = func.get(vm)?;
    
    let args = vm.get_call_args();
    let this = vm.get_this()?;
    
    let ret = func.call(vm.get_realm(), args, this)?;
    
    output.set(ret, vm)
}

pub fn call_member(obj: impl Data, member: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let obj = obj.get(vm)?;
    let member = member.get(vm)?;
    
    let args = vm.get_call_args();
    
    let ret = obj.call_method(&member, vm.get_realm(), args)?;
    
    output.set(ret, vm)
}