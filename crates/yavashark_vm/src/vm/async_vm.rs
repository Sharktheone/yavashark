use crate::execute::Execute;
use crate::{Registers, Stack, VM};
use std::mem;
use std::rc::Rc;
use yavashark_bytecode::data::{Label, OutputData, OutputDataType};
use yavashark_bytecode::{BytecodeFunctionCode, ConstIdx, Reg, VarName};
use yavashark_env::scope::Scope;
use yavashark_env::{ControlFlow, Error, ObjectHandle, Realm, Res, Value};
use crate::consts::ConstIntoValue;

pub struct VmState {
    regs: Registers,
    stack: Stack,
    call_args: Vec<Value>,

    pc: usize,
    code: Rc<BytecodeFunctionCode>,

    current_scope: Scope,

    pub acc: Value,

    continue_storage: Option<OutputDataType>,
}

pub struct AsyncVM<'a> {
    state: VmState,
    realm: &'a mut Realm,
}

pub enum AsyncPoll {
    Await(VmState, ObjectHandle),
    Ret(VmState, Res),
}

impl VmState {
    pub const fn new(code: Rc<BytecodeFunctionCode>, scope: Scope) -> Self {
        Self {
            regs: Registers::new(),
            stack: Stack::new(),
            call_args: Vec::new(),
            pc: 0,
            code,
            current_scope: scope,
            acc: Value::Undefined,
            continue_storage: None,
        }
    }
    
    pub fn continue_async(&mut self, val: Value) -> Res {
        if let Some(storage) = self.continue_storage.take() {
            match storage {
                OutputDataType::Acc(_) => self.acc = val,
                OutputDataType::Reg(reg) => self.regs.set(reg.0, val)?,
                OutputDataType::Stack(stack) => self.stack.set(stack.0 as usize, val),
                OutputDataType::Var(name) => {
                    let name = self
                        .code
                        .ds
                        .var_names
                        .get(name.0 as usize)
                        .map(String::as_str)
                        .ok_or(Error::reference("Invalid variable name"))?;
                    
                    self.current_scope.declare_var(name.into(), val)?;
                }
                
                
            }
        }
        
        Ok(())
    }

}

impl<'a> AsyncVM<'a> {
    #[must_use]
    pub const fn new(code: Rc<BytecodeFunctionCode>, scope: Scope, realm: &'a mut Realm) -> Self {
        let state = VmState::new(code, scope);

        Self { state, realm }
    }

    #[must_use]
    pub const fn from_state(state: VmState, realm: &'a mut Realm) -> Self {
        Self { state, realm }
    }

    pub fn run(mut self) -> AsyncPoll {
        while self.state.pc < self.state.code.instructions.len() {
            let instr = &self.state.code.instructions[self.state.pc];
            self.state.pc += 1;

            match instr.execute(&mut self) {
                Ok(_) => {}
                Err(e) => match e {
                    ControlFlow::Error(e) => return AsyncPoll::Ret(self.state, Err(e)),
                    ControlFlow::Return(value) => {
                        self.state.acc = value;
                        break;
                    }
                    ControlFlow::Break(_) => {
                        return AsyncPoll::Ret(
                            self.state,
                            Err(Error::new("Break outside of loop")),
                        );
                    }
                    ControlFlow::Continue(_) => {
                        return AsyncPoll::Ret(
                            self.state,
                            Err(Error::new("Continue outside of loop")),
                        );
                    }
                    ControlFlow::Await(out) => {
                        return AsyncPoll::Await(self.state, out);
                    }
                    ControlFlow::Yield(_) => {
                        return AsyncPoll::Ret(
                            self.state,
                            Err(Error::new("Yield outside of generator")),
                        );
                    }
                    ControlFlow::OptChainShortCircuit => {}
                },
            }
        }

        AsyncPoll::Ret(self.state, Ok(()))
    }
}

impl<'a> VM for AsyncVM<'a> {
    fn acc(&self) -> Value {
        self.state.acc.clone()
    }

    fn set_acc(&mut self, value: Value) {
        self.state.acc = value;
    }

    fn get_variable(&mut self, name: VarName) -> Res<Value> {
        let Some(name) = self.state.code.ds.var_names.get(name as usize) else {
            return Err(Error::reference("Invalid variable name"));
        };

        self.state
            .current_scope
            .resolve(name)?
            .ok_or(Error::reference("Variable not found"))
    }

    #[must_use]
    fn var_name(&self, name: VarName) -> Option<&str> {
        self.state
            .code
            .ds
            .var_names
            .get(name as usize)
            .map(String::as_str)
    }

    fn get_register(&self, reg: Reg) -> Res<Value> {
        self.state
            .regs
            .get(reg)
            .ok_or(Error::reference("Invalid register"))
    }

    fn get_label(&self, label: Label) -> Res<&str> {
        self.state
            .code
            .ds
            .labels
            .get(label.0 as usize)
            .map(String::as_str)
            .ok_or(Error::reference("Invalid label"))
    }

    fn set_variable(&mut self, name: VarName, value: Value) -> Res {
        let name = self
            .var_name(name)
            .ok_or(Error::reference("Invalid variable name"))?;
        self.state.current_scope.declare_var(name.into(), value)
    }

    fn set_register(&mut self, reg: Reg, value: Value) -> Res {
        self.state.regs.set(reg, value)
    }

    fn push(&mut self, value: Value) {
        self.state.stack.push(value);
    }

    fn pop(&mut self) -> Option<Value> {
        self.state.stack.pop()
    }

    fn set_accb(&mut self, value: bool) {
        self.state.acc = Value::Boolean(value);
    }

    fn get_this(&self) -> Res<Value> {
        self.state.current_scope.this()
    }

    fn get_constant(&self, const_idx: ConstIdx) -> Res<Value> {
        let val = self
            .state
            .code
            .ds
            .constants
            .get(const_idx as usize)
            .ok_or(Error::reference("Invalid constant index"))?;

        val.clone().into_value(&self.realm, &self.state.current_scope)
    }

    #[must_use]
    fn get_stack(&self, idx: u32) -> Option<Value> {
        self.state.stack.get(idx as usize).cloned()
    }

    fn set_stack(&mut self, idx: u32, value: Value) -> Res {
        self.state.stack.set(idx as usize, value);

        Ok(())
    }

    fn get_args(&mut self, num: u16) -> Vec<Value> {
        self.state.stack.pop_n(num as usize)
    }

    fn get_realm(&mut self) -> &mut Realm {
        &mut self.realm
    }

    fn set_pc(&mut self, pc: usize) {
        self.state.pc = pc;
    }

    fn offset_pc(&mut self, offset: isize) {
        // pc won't be above isize::MAX, since this is `Vec`'s length limit
        self.state.pc = (self.state.pc as isize + offset) as usize;
    }

    fn push_scope(&mut self) -> Res {
        self.state.current_scope = self.state.current_scope.child()?;

        Ok(())
    }

    fn pop_scope(&mut self) -> Res {
        let scope = self.state.current_scope.parent()?;

        if let Some(p) = scope {
            self.state.current_scope = p.into();
        } else {
            return Err(Error::new("No parent scope"));
        }

        Ok(())
    }

    fn push_call_args(&mut self, args: Vec<Value>) {
        self.state.call_args.extend(args);
    }

    fn push_call_arg(&mut self, arg: Value) {
        self.state.call_args.push(arg);
    }

    fn get_call_args(&mut self) -> Vec<Value> {
        mem::take(&mut self.state.call_args)
    }

    fn get_scope(&self) -> &Scope {
        &self.state.current_scope
    }

    fn get_scope_mut(&mut self) -> &mut Scope {
        &mut self.state.current_scope
    }

    fn set_continue_storage(&mut self, out: impl OutputData) {
        self.state.continue_storage = Some(out.data_type());
    }
}
