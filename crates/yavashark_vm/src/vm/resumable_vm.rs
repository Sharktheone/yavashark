use crate::consts::ConstIntoValue;
use crate::execute::Execute;
use crate::{Registers, Stack, VM};
use std::mem;
use std::rc::Rc;
use yavashark_bytecode::control::{ControlBlock, TryBlock};
use yavashark_bytecode::data::{ControlIdx, DataSection, Label, OutputData, OutputDataType};
use yavashark_bytecode::{instructions, BytecodeFunctionCode, ConstIdx, Reg, VarName};
use yavashark_env::error_obj::ErrorObj;
use yavashark_env::scope::Scope;
use yavashark_env::value::property_key::IntoPropertyKey;
use yavashark_env::{
    ControlFlow, Error, Object, ObjectHandle, PropertyKey, Realm, Res, Value, ValueResult,
};

#[derive(Debug, Clone)]
pub struct VmState<T: VMStateFunctionCode = Rc<BytecodeFunctionCode>> {
    pub regs: Registers,
    pub stack: Stack,
    pub call_args: Vec<Value>,

    pub pc: usize,
    pub code: T,

    pub current_scope: Scope,

    pub acc: Value,

    pub continue_storage: Option<OutputDataType>,

    pub try_stack: Vec<TryBlock>,
    pub yield_star_val: Option<ObjectHandle>,

    pub spread_stack: Vec<Vec<PropertyKey>>,
    pub throw: Option<Error>,
}

pub trait VMStateFunctionCode: Clone {
    fn instructions(&self) -> &[instructions::Instruction];
    fn data_section(&self) -> &DataSection;
}

impl VMStateFunctionCode for Rc<BytecodeFunctionCode> {
    fn instructions(&self) -> &[instructions::Instruction] {
        &self.instructions
    }

    fn data_section(&self) -> &DataSection {
        &self.ds
    }
}

pub struct ResumableVM<'a, T: VMStateFunctionCode = Rc<BytecodeFunctionCode>> {
    state: VmState<T>,
    realm: &'a mut Realm,
}

pub enum AsyncPoll {
    Await(VmState, ObjectHandle),
    Ret(VmState, Res),
}

#[allow(clippy::large_enum_variant)]
pub enum GeneratorPoll {
    Ret(ValueResult),
    Yield(VmState, Value),
}

pub enum AsyncGeneratorPoll {
    Await(VmState, ObjectHandle),
    Ret(VmState, ValueResult),
    Yield(VmState, Value),
}

impl<T: VMStateFunctionCode> VmState<T> {
    #[must_use]
    pub const fn new(code: T, scope: Scope) -> Self {
        Self {
            regs: Registers::new(),
            stack: Stack::new(),
            call_args: Vec::new(),
            pc: 0,
            code,
            current_scope: scope,
            acc: Value::Undefined,
            continue_storage: None,
            spread_stack: Vec::new(),
            try_stack: Vec::new(),
            yield_star_val: None,
            throw: None,
        }
    }
    pub fn continue_async(&mut self, val: Value, realm: &mut Realm) -> Res {
        if let Some(storage) = self.continue_storage.take() {
            match storage {
                OutputDataType::Acc(_) => self.acc = val,
                OutputDataType::Reg(reg) => self.regs.set(reg.0, val)?,
                OutputDataType::Stack(stack) => self.stack.set(stack.0 as usize, val),
                OutputDataType::Var(name) => {
                    let name = self
                        .code
                        .data_section()
                        .var_names
                        .get(name.0 as usize)
                        .map(String::as_str)
                        .ok_or(Error::reference("Invalid variable name"))?;

                    self.current_scope.declare_var(name.into(), val, realm)?;
                }
            }
        }

        Ok(())
    }
}

impl<'a, T: VMStateFunctionCode> ResumableVM<'a, T> {
    #[must_use]
    pub const fn new(code: T, scope: Scope, realm: &'a mut Realm) -> Self {
        let state = VmState::new(code, scope);

        Self { state, realm }
    }

    #[must_use]
    pub const fn from_state(state: VmState<T>, realm: &'a mut Realm) -> Self {
        Self { state, realm }
    }

    pub fn handle_root_error(&mut self, err: Error) -> Res {
        if self.state.try_stack.is_empty() {
            return Err(err);
        }

        self.handle_error(err)
    }

    pub fn handle_error(&mut self, err: Error) -> Res {
        if let Some(tb) = self.state.try_stack.last_mut() {
            if let Some(catch) = tb.catch.take() {
                self.set_pc(catch);
                let error = ErrorObj::error_to_value(err, self.realm)?;

                self.set_acc(error);
            } else if let Some(finally) = tb.finally.take() {
                self.state.throw = Some(err);
                self.set_pc(finally);

                self.state.try_stack.pop();
            }
        }

        Ok(())
    }
}

impl ResumableVM<'_> {
    #[must_use]
    pub fn next(mut self) -> GeneratorPoll {
        match self.yield_star_checkpoint() {
            Some(Ok(next)) => {
                return GeneratorPoll::Yield(self.state, next);
            }
            Some(Err(e)) => {
                return GeneratorPoll::Ret(Err(e));
            }
            None => {}
        }

        while self.state.pc < self.state.code.instructions.len() {
            let instr = &self.state.code.instructions[self.state.pc];
            self.state.pc += 1;

            match instr.execute(&mut self) {
                Ok(()) => {}
                Err(e) => match e {
                    ControlFlow::Error(e) => {
                        if let Err(e) = self.handle_root_error(e) {
                            return GeneratorPoll::Ret(Err(e));
                        }

                        return self.next();
                    }
                    ControlFlow::Return(value) => return GeneratorPoll::Ret(Ok(value)),
                    ControlFlow::Break(_) => {
                        return GeneratorPoll::Ret(Err(Error::new("Break outside of loop")));
                    }
                    ControlFlow::Continue(_) => {
                        return GeneratorPoll::Ret(Err(Error::new("Continue outside of loop")));
                    }
                    ControlFlow::Await(_) => {
                        return GeneratorPoll::Ret(Err(Error::new("Await outside async function")));
                    }
                    ControlFlow::Yield(v) => {
                        return GeneratorPoll::Yield(self.state, v);
                    }
                    ControlFlow::YieldStar(i) => {
                        let ys = match i.get_iter(self.realm).and_then(Value::to_object) {
                            Ok(o) => o,
                            Err(e) => {
                                if let Err(e) = self.handle_root_error(e) {
                                    return GeneratorPoll::Ret(Err(e));
                                }

                                return self.next();
                            }
                        };

                        return match ys.iter_next(self.realm) {
                            Ok(Some(next)) => {
                                self.state.yield_star_val = Some(ys);
                                GeneratorPoll::Yield(self.state, next)
                            }
                            Ok(None) => self.next(),
                            Err(e) => {
                                if let Err(e) = self.handle_root_error(e) {
                                    return GeneratorPoll::Ret(Err(e));
                                }

                                self.next()
                            }
                        };
                    }
                    ControlFlow::OptChainShortCircuit => {}
                },
            }
        }

        GeneratorPoll::Ret(Ok(Value::Undefined))
    }

    #[must_use]
    pub fn poll_next(mut self) -> AsyncGeneratorPoll {
        match self.yield_star_checkpoint() {
            Some(Ok(next)) => {
                return AsyncGeneratorPoll::Yield(self.state, next);
            }
            Some(Err(e)) => {
                return AsyncGeneratorPoll::Ret(self.state, Err(e));
            }
            None => {}
        }

        while self.state.pc < self.state.code.instructions.len() {
            let instr = &self.state.code.instructions[self.state.pc];
            self.state.pc += 1;

            match instr.execute(&mut self) {
                Ok(()) => {}
                Err(e) => match e {
                    ControlFlow::Error(e) => {
                        if let Err(e) = self.handle_root_error(e) {
                            return AsyncGeneratorPoll::Ret(self.state, Err(e));
                        }

                        return self.poll_next();
                    }
                    ControlFlow::Return(value) => {
                        return AsyncGeneratorPoll::Ret(self.state, Ok(value))
                    }
                    ControlFlow::Break(_) => {
                        return AsyncGeneratorPoll::Ret(
                            self.state,
                            Err(Error::new("Break outside of loop")),
                        );
                    }
                    ControlFlow::Continue(_) => {
                        return AsyncGeneratorPoll::Ret(
                            self.state,
                            Err(Error::new("Continue outside of loop")),
                        );
                    }
                    ControlFlow::Await(out) => {
                        return AsyncGeneratorPoll::Await(self.state, out);
                    }
                    ControlFlow::Yield(v) => {
                        return AsyncGeneratorPoll::Yield(self.state, v);
                    }
                    ControlFlow::YieldStar(i) => {
                        let ys = match i.get_iter(self.realm).and_then(Value::to_object) {
                            Ok(o) => o,
                            Err(e) => {
                                if let Err(e) = self.handle_root_error(e) {
                                    return AsyncGeneratorPoll::Ret(self.state, Err(e));
                                }

                                return self.poll_next();
                            }
                        };

                        return match ys.iter_next(self.realm) {
                            Ok(Some(next)) => {
                                self.state.yield_star_val = Some(ys);
                                AsyncGeneratorPoll::Yield(self.state, next)
                            }
                            Ok(None) => self.poll_next(),
                            Err(e) => {
                                if let Err(e) = self.handle_root_error(e) {
                                    return AsyncGeneratorPoll::Ret(self.state, Err(e));
                                }

                                self.poll_next()
                            }
                        };
                    }
                    ControlFlow::OptChainShortCircuit => {}
                },
            }
        }

        AsyncGeneratorPoll::Ret(self.state, Ok(Value::Undefined))
    }

    #[must_use]
    pub fn poll(mut self) -> AsyncPoll {
        while self.state.pc < self.state.code.instructions.len() {
            let instr = &self.state.code.instructions[self.state.pc];
            self.state.pc += 1;

            match instr.execute(&mut self) {
                Ok(()) => {}
                Err(e) => match e {
                    ControlFlow::Error(e) => {
                        if let Err(e) = self.handle_root_error(e) {
                            return AsyncPoll::Ret(self.state, Err(e));
                        }

                        return self.poll();
                    }
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
                    ControlFlow::Yield(_) | ControlFlow::YieldStar(_) => {
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

    pub fn yield_star_checkpoint(&mut self) -> Option<Res<Value>> {
        let ys = self.state.yield_star_val.as_ref()?;

        match ys.iter_next(self.realm) {
            Ok(Some(next)) => Some(Ok(next)),
            Ok(None) => {
                self.state.yield_star_val = None;
                None
            }
            Err(e) => Some(Err(e)),
        }
    }
}

impl<T: VMStateFunctionCode> VM for ResumableVM<'_, T> {
    fn acc(&self) -> Value {
        self.state.acc.clone()
    }

    fn set_acc(&mut self, value: Value) {
        self.state.acc = value;
    }

    fn get_variable(&mut self, name: VarName) -> Res<Value> {
        let Some(name) = self.state.code.data_section().var_names.get(name as usize) else {
            return Err(Error::reference("Invalid variable name"));
        };

        self.state
            .current_scope
            .resolve(name, self.realm)?
            .ok_or(Error::reference("Variable not found"))
    }

    fn var_name(&self, name: VarName) -> Option<&str> {
        self.state
            .code
            .data_section()
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
            .data_section()
            .labels
            .get(label.0 as usize)
            .map(String::as_str)
            .ok_or(Error::reference("Invalid label"))
    }

    fn set_variable(&mut self, name: VarName, value: Value) -> Res {
        let name = self
            .var_name(name)
            .ok_or(Error::reference("Invalid variable name"))?;
        self.state
            .current_scope
            .update_or_define(name.into(), value, self.realm)
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

    fn get_constant(&mut self, const_idx: ConstIdx) -> Res<Value> {
        let val = self
            .state
            .code
            .data_section()
            .constants
            .get(const_idx as usize)
            .ok_or(Error::reference("Invalid constant index"))?;

        val.clone().into_value(self)
    }

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
        self.realm
    }

    fn get_realm_ref(&self) -> &Realm {
        self.realm
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

    fn enter_try(&mut self, id: ControlIdx) -> Res {
        let Some(c) = self.state.code.data_section().control.get(id.0 as usize) else {
            return Err(Error::new("Invalid control index"));
        };

        let ControlBlock::Try(tb) = c else {
            return Err(Error::new("Control block is not a try block"));
        };

        self.state.try_stack.push(*tb);

        Ok(())
    }

    fn leave_try(&mut self) -> Res {
        let tb = self
            .state
            .try_stack
            .last_mut()
            .ok_or(Error::new("No try block"))?;

        if let Some(f) = tb.finally.take() {
            self.set_pc(f);
        } else {
            let exit = tb.exit;
            if let Some(err) = self.state.throw.take() {
                return self.handle_error(err);
            }

            self.set_pc(exit);
            self.state.try_stack.pop();
        }

        Ok(())
    }

    fn begin_spread(&mut self, cap: usize) -> Res {
        self.state.spread_stack.push(Vec::with_capacity(cap));

        Ok(())
    }

    fn push_spread(&mut self, elem: Value) -> Res {
        let Some(last) = self.state.spread_stack.last_mut() else {
            return Err(Error::new("No spread in progress"));
        };

        last.push(elem.into_property_key(self.realm)?);

        Ok(())
    }

    fn end_spread(&mut self, obj: ObjectHandle) -> Res<ObjectHandle> {
        let not = self
            .state
            .spread_stack
            .pop()
            .ok_or(Error::new("No spread in progress"))?;

        let mut props = Vec::new();

        for (name, value) in obj.properties(self.get_realm())? {
            if !not.contains(&name) {
                props.push((name, value));
            }
        }

        let rest_obj = Object::from_values(props, self.get_realm())?;

        Ok(rest_obj)
    }

    fn end_spread_no_output(&mut self) -> Res {
        let _ = self
            .state
            .spread_stack
            .pop()
            .ok_or(Error::new("No spread in progress"))?;

        Ok(())
    }
}
