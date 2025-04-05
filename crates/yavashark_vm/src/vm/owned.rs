use crate::execute::Execute;
use crate::{Registers, Stack, VM};
use std::mem;
use std::path::PathBuf;
use yavashark_bytecode::data::{DataSection, Label, OutputData, OutputDataType};
use yavashark_bytecode::instructions::Instruction;
use yavashark_bytecode::{ConstIdx, Reg, VarName};
use yavashark_env::scope::Scope;
use yavashark_env::{Error, Realm, Res, Value};
use crate::consts::ConstIntoValue;

pub struct OwnedVM {
    regs: Registers,
    stack: Stack,
    call_args: Vec<Value>,

    pc: usize,
    code: Vec<Instruction>,
    data: DataSection,

    current_scope: Scope,

    acc: Value,

    realm: Realm,
    continue_storage: Option<OutputDataType>,
}

impl OwnedVM {
    pub fn new(
        code: Vec<Instruction>,
        data: DataSection,
        file: PathBuf,
    ) -> yavashark_env::Res<Self> {
        let realm = Realm::new()?;

        Ok(Self {
            regs: Registers::new(),
            stack: Stack::new(),
            call_args: Vec::new(),
            pc: 0,
            code,
            data,
            current_scope: Scope::new(&realm, file),
            acc: Value::Undefined,
            realm,
            continue_storage: None,
        })
    }

    #[must_use]
    pub fn with_realm(
        code: Vec<Instruction>,
        data: DataSection,
        realm: Realm,
        file: PathBuf,
    ) -> Self {
        Self {
            regs: Registers::new(),
            stack: Stack::new(),
            call_args: Vec::new(),
            pc: 0,
            code,
            data,
            current_scope: Scope::new(&realm, file),
            acc: Value::Undefined,
            realm,
            continue_storage: None,
        }
    }

    #[must_use]
    pub const fn with_realm_scope(
        code: Vec<Instruction>,
        data: DataSection,
        realm: Realm,
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
        }
    }

    pub fn run(&mut self) -> Res {
        while self.pc < self.code.len() {
            let instr = self.code[self.pc];
            self.pc += 1;

            instr.execute(self)?;
        }

        Ok(())
    }
}

impl VM for OwnedVM {
    fn acc(&self) -> Value {
        self.acc.clone()
    }

    fn set_acc(&mut self, value: Value) {
        self.acc = value;
    }

    fn get_variable(&mut self, name: VarName) -> Res<Value> {
        let Some(name) = self.data.var_names.get(name as usize) else {
            return Err(Error::reference("Invalid variable name"));
        };

        self.current_scope
            .resolve(name)?
            .ok_or(Error::reference("Variable not found"))
    }

    #[must_use]
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
        self.current_scope.declare_var(name.into(), value)
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

        val.clone().into_value(&self.realm, &self.current_scope)
    }

    #[must_use]
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
        &mut self.realm
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
}
