use crate::consts::ConstIntoValue;
use crate::execute::Execute;
use crate::{Registers, Stack, VM};
use std::mem;
use std::path::PathBuf;
use yavashark_bytecode::control::{ControlBlock, TryBlock};
use yavashark_bytecode::data::{ControlIdx, DataSection, Label, OutputData, OutputDataType};
use yavashark_bytecode::instructions::Instruction;
use yavashark_bytecode::{ConstIdx, Reg, VarName};
use yavashark_env::error::ErrorObj;
use yavashark_env::scope::Scope;
use yavashark_env::{ControlResult, Error, Object, ObjectHandle, Realm, Res, Value};

pub struct BorrowedVM<'a> {
    regs: Registers,
    stack: Stack,
    call_args: Vec<Value>,

    pc: usize,
    code: &'a [Instruction],
    data: &'a DataSection,

    pub current_scope: Scope,

    acc: Value,

    realm: &'a mut Realm,

    continue_storage: Option<OutputDataType>,

    spread_stack: Vec<Vec<Value>>,
    try_stack: Vec<TryBlock>,

    throw: Option<Error>,
}

impl<'a> BorrowedVM<'a> {
    pub fn new(
        code: &'a [Instruction],
        data: &'a DataSection,
        realm: &'a mut Realm,
        file: PathBuf,
    ) -> Res<Self> {
        Ok(Self {
            regs: Registers::new(),
            stack: Stack::new(),
            call_args: Vec::new(),
            pc: 0,
            code,
            data,
            current_scope: Scope::new(realm, file),
            acc: Value::Undefined,
            realm,
            continue_storage: None,
            spread_stack: Vec::new(),
            try_stack: Vec::new(),
            throw: None,
        })
    }

    #[must_use]
    pub const fn with_scope(
        code: &'a [Instruction],
        data: &'a DataSection,
        realm: &'a mut Realm,
        scope: Scope,
    ) -> Self {
        Self {
            regs: Registers::new(),
            stack: Stack::new(),
            call_args: Vec::new(),
            pc: 0,
            code,
            data,
            current_scope: scope,
            acc: Value::Undefined,
            realm,
            continue_storage: None,
            spread_stack: Vec::new(),
            try_stack: Vec::new(),
            throw: None,
        }
    }

    pub fn run(&mut self) -> ControlResult {
        while self.pc < self.code.len() {
            let instr = self.code[self.pc];
            self.pc += 1;

            instr.execute(self)?;
        }

        Ok(())
    }

    pub fn handle_error(&mut self, err: Error) -> Res {
        if let Some(tb) = self.try_stack.last_mut() {
            if let Some(catch) = tb.catch.take() {
                if tb.finally.is_none() {
                    self.try_stack.pop();
                }
                self.offset_pc(catch);
                self.set_acc(ErrorObj::error_to_value(err, self.realm));
            } else if let Some(finally) = tb.finally.take() {
                self.throw = Some(err);
                self.offset_pc(finally);

                self.try_stack.pop();
            }
        }

        Ok(())
    }
}

impl VM for BorrowedVM<'_> {
    fn acc(&self) -> Value {
        self.acc.clone()
    }

    fn set_acc(&mut self, value: Value) {
        self.acc = value;
    }

    fn get_variable(&self, name: VarName) -> Res<Value> {
        let Some(name) = self.data.var_names.get(name as usize) else {
            return Err(Error::reference("Invalid variable name"));
        };

        self.current_scope
            .resolve(name)?
            .ok_or(Error::reference("Variable not found"))
    }

    fn var_name(&self, name: VarName) -> Option<&str> {
        self.data.var_names.get(name as usize).map(String::as_str)
    }

    fn get_register(&self, reg: Reg) -> Res<Value> {
        self.regs
            .get(reg)
            .ok_or(Error::reference("Invalid register"))
    }

    fn get_label(&self, label: Label) -> Res<&str> {
        self.data
            .labels
            .get(label.0 as usize)
            .map(String::as_str)
            .ok_or(Error::reference("Invalid label"))
    }

    fn set_variable(&mut self, name: VarName, value: Value) -> Res {
        let name = self
            .var_name(name)
            .ok_or(Error::reference("Invalid variable name"))?;
        self.current_scope.update_or_define(name.into(), value)
    }

    fn set_register(&mut self, reg: Reg, value: Value) -> Res {
        self.regs.set(reg, value)
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Option<Value> {
        self.stack.pop()
    }

    fn set_accb(&mut self, value: bool) {
        self.acc = Value::Boolean(value);
    }

    fn get_this(&self) -> Res<Value> {
        self.current_scope.this()
    }

    fn get_constant(&self, const_idx: ConstIdx) -> Res<Value> {
        let val = self
            .data
            .constants
            .get(const_idx as usize)
            .ok_or(Error::reference("Invalid constant index"))?;

        val.clone().into_value(self)
    }

    fn get_stack(&self, idx: u32) -> Option<Value> {
        self.stack.get(idx as usize).cloned()
    }

    fn set_stack(&mut self, idx: u32, value: Value) -> Res {
        self.stack.set(idx as usize, value);

        Ok(())
    }

    fn get_args(&mut self, num: u16) -> Vec<Value> {
        self.stack.pop_n(num as usize)
    }

    fn get_realm(&mut self) -> &mut Realm {
        self.realm
    }

    fn get_realm_ref(&self) -> &Realm {
        self.realm
    }

    fn set_pc(&mut self, pc: usize) {
        self.pc = pc;
    }

    fn offset_pc(&mut self, offset: isize) {
        // pc won't be above isize::MAX, since this is `Vec`'s length limit
        self.pc = (self.pc as isize + offset) as usize;
    }

    fn push_scope(&mut self) -> Res {
        self.current_scope = self.current_scope.child()?;

        Ok(())
    }

    fn pop_scope(&mut self) -> Res {
        let scope = self.current_scope.parent()?;

        if let Some(p) = scope {
            self.current_scope = p.into();
        } else {
            return Err(Error::new("No parent scope"));
        }

        Ok(())
    }

    fn push_call_args(&mut self, args: Vec<Value>) {
        self.call_args.extend(args);
    }

    fn push_call_arg(&mut self, arg: Value) {
        self.call_args.push(arg);
    }

    fn get_call_args(&mut self) -> Vec<Value> {
        mem::take(&mut self.call_args)
    }

    fn get_scope(&self) -> &Scope {
        &self.current_scope
    }

    fn get_scope_mut(&mut self) -> &mut Scope {
        &mut self.current_scope
    }

    fn set_continue_storage(&mut self, out: impl OutputData) {
        self.continue_storage = Some(out.data_type());
    }

    fn enter_try(&mut self, id: ControlIdx) -> Res {
        let Some(c) = self.data.control.get(id.0 as usize) else {
            return Err(Error::new("Invalid control index"));
        };

        let ControlBlock::Try(tb) = c else {
            return Err(Error::new("Control block is not a try block"));
        };

        self.try_stack.push(*tb);

        Ok(())
    }

    fn leave_try(&mut self) -> Res {
        let tb = self
            .try_stack
            .last_mut()
            .ok_or(Error::new("No try block"))?;

        if let Some(f) = tb.finally.take() {
            self.offset_pc(f);
        } else {
            let exit = tb.exit;

            if let Some(err) = self.throw.take() {
                return self.handle_error(err);
            }

            self.offset_pc(exit);
            self.try_stack.pop();
        }

        Ok(())
    }

    fn begin_spread(&mut self, cap: usize) -> Res {
        self.spread_stack.push(Vec::with_capacity(cap));

        Ok(())
    }

    fn push_spread(&mut self, elem: Value) -> Res {
        let Some(last) = self.spread_stack.last_mut() else {
            return Err(Error::new("No spread in progress"));
        };

        last.push(elem);

        Ok(())
    }

    fn end_spread(&mut self, obj: ObjectHandle) -> Res<ObjectHandle> {
        let not= self
            .spread_stack
            .pop()
            .ok_or(Error::new("No spread in progress"))?;

        let mut props = Vec::new();

        for (name, value) in obj.properties()? {
            if !not.contains(&name) {
                props.push((name, value));
            }
        }


        let rest_obj = Object::from_values(props, self.get_realm())?;

        Ok(rest_obj)
    }

    fn end_spread_no_output(&mut self) -> Res {
        let _ = self
            .spread_stack
            .pop()
            .ok_or(Error::new("No spread in progress"))?;

        Ok(())
    }
}
