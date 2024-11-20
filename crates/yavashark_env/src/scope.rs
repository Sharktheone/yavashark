use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use yavashark_garbage::collectable::CellCollectable;
use yavashark_garbage::{Collectable, Gc, GcRef};
use yavashark_value::CustomGcRefUntyped;

use crate::console::get_console;
use crate::realm::Realm;
use crate::{Error, ObjectHandle, Res, Result, Value, Variable};

pub struct MutValue {
    pub name: String,
    pub scope: Rc<RefCell<ScopeInternal>>,
}

#[derive(Debug, Clone, Default)]
#[allow(clippy::module_name_repetitions)]
pub struct ScopeState {
    state: u8,
}

#[allow(unused)]
impl ScopeState {
    const NONE: u8 = 0b0;
    const FUNCTION: u8 = 0b10;
    const ITERATION: u8 = 0b100;
    const BREAKABLE: u8 = 0b1000;
    const RETURNABLE: u8 = 0b10000;
    const CONTINUABLE: u8 = 0b10_0000;
    const OPT_CHAIN: u8 = 0b100_0000;
    const STATE_NONE: Self = Self { state: Self::NONE };
    const STATE_FUNCTION: Self = Self {
        state: Self::FUNCTION,
    };
    const STATE_ITERATION: Self = Self {
        state: Self::ITERATION,
    };
    const STATE_BREAKABLE: Self = Self {
        state: Self::BREAKABLE,
    };
    const STATE_RETURNABLE: Self = Self {
        state: Self::RETURNABLE,
    };

    const STATE_CONTINUABLE: Self = Self {
        state: Self::CONTINUABLE,
    };

    const STATE_OPT_CHAIN: Self = Self {
        state: Self::OPT_CHAIN,
    };

    #[must_use]
    pub const fn new() -> Self {
        Self { state: 0 }
    }

    #[must_use]
    pub const fn copy(&self) -> Self {
        let mut state = self.state;

        state &= !Self::FUNCTION; // Remove the function state

        Self { state }
    }

    pub fn set_function(&mut self) {
        self.state |= Self::FUNCTION;
        self.state |= Self::RETURNABLE;
    }

    pub fn set_iteration(&mut self) {
        self.state |= Self::ITERATION;
        self.state |= Self::BREAKABLE;
    }

    pub fn set_breakable(&mut self) {
        self.state |= Self::BREAKABLE;
    }

    pub fn set_returnable(&mut self) {
        self.state |= Self::RETURNABLE;
    }

    pub fn set_loop(&mut self) {
        self.state |= Self::CONTINUABLE;
        self.state |= Self::BREAKABLE;
    }

    pub fn set_opt_chain(&mut self) {
        self.state |= Self::OPT_CHAIN;
    }

    #[must_use]
    pub const fn is_function(&self) -> bool {
        self.state & Self::FUNCTION != 0
    }

    #[must_use]
    pub const fn is_iteration(&self) -> bool {
        self.state & Self::ITERATION != 0
    }

    #[must_use]
    pub const fn is_breakable(&self) -> bool {
        self.state & Self::BREAKABLE != 0
    }

    #[must_use]
    pub const fn is_returnable(&self) -> bool {
        self.state & Self::RETURNABLE != 0
    }

    #[must_use]
    pub const fn is_none(&self) -> bool {
        self.state == Self::NONE
    }

    #[must_use]
    pub const fn is_continuable(&self) -> bool {
        self.state & Self::CONTINUABLE != 0
    }
    #[must_use]
    pub const fn is_opt_chain(&self) -> bool {
        self.state & Self::OPT_CHAIN != 0
    }
}

#[derive(Debug, Clone)]
pub struct Scope {
    scope: Gc<RefCell<ScopeInternal>>,
}

#[derive(Debug)]
pub enum ParentOrGlobal {
    Parent(Gc<RefCell<ScopeInternal>>),
    Global(ObjectHandle),
}

impl Clone for ParentOrGlobal {
    fn clone(&self) -> Self {
        match self {
            Self::Parent(p) => Self::Parent(p.clone()),
            Self::Global(g) => Self::Global(g.clone()),
        }
    }
}

impl ParentOrGlobal {
    fn get_ref(&self) -> GcRef<RefCell<ScopeInternal>> {
        match self {
            Self::Parent(p) => p.get_ref(),
            Self::Global(g) => g.gc_get_untyped_ref(),
        }
    }
}

#[derive(Debug)]
pub struct ScopeInternal {
    parent: ParentOrGlobal,
    variables: HashMap<String, Variable>,
    pub available_labels: Vec<String>,
    pub last_label_is_current: bool,
    pub state: ScopeState,
    pub this: Value,
    pub file: Option<PathBuf>,
}

unsafe impl CellCollectable<RefCell<Self>> for ScopeInternal {
    fn get_refs(&self) -> Vec<GcRef<RefCell<Self>>> {
        let mut refs = Vec::with_capacity(self.variables.len());

        for v in self.variables.values() {
            if let Value::Object(o) = &v.value {
                refs.push(o.gc_get_untyped_ref());
            }
        }

        refs.push(self.parent.get_ref());

        if let Some(this) = self.this.gc_untyped_ref() {
            refs.push(this);
        }

        refs
    }
}

impl ScopeInternal {
    #[must_use]
    pub fn new(realm: &Realm, path: PathBuf) -> Self {
        let mut variables = HashMap::with_capacity(8);

        variables.insert(
            "undefined".to_string(),
            Variable::new_read_only(Value::Undefined),
        );
        variables.insert(
            "NaN".to_string(),
            Variable::new_read_only(Value::Number(f64::NAN)),
        );
        variables.insert(
            "Infinity".to_string(),
            Variable::new_read_only(Value::Number(f64::INFINITY)),
        );
        variables.insert("null".to_string(), Variable::new_read_only(Value::Null));
        variables.insert(
            "true".to_string(),
            Variable::new_read_only(Value::Boolean(true)),
        );
        variables.insert(
            "false".to_string(),
            Variable::new_read_only(Value::Boolean(false)),
        );
        variables.insert(
            "console".to_string(),
            Variable::new_read_only(get_console(realm)),
        );
        Self {
            parent: ParentOrGlobal::Global(realm.global.clone()),
            variables,
            available_labels: Vec::new(),
            last_label_is_current: false,
            state: ScopeState::new(),
            this: Value::Undefined,
            file: Some(path),
        }
    }

    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn global(realm: &Realm, path: PathBuf) -> Self {
        let mut variables = HashMap::with_capacity(8);

        variables.insert(
            "undefined".to_string(),
            Variable::new_read_only(Value::Undefined),
        );
        variables.insert(
            "NaN".to_string(),
            Variable::new_read_only(Value::Number(f64::NAN)),
        );
        variables.insert(
            "Infinity".to_string(),
            Variable::new_read_only(Value::Number(f64::INFINITY)),
        );
        variables.insert("null".to_string(), Variable::new_read_only(Value::Null));
        variables.insert(
            "true".to_string(),
            Variable::new_read_only(Value::Boolean(true)),
        );
        variables.insert(
            "false".to_string(),
            Variable::new_read_only(Value::Boolean(false)),
        );

        Self {
            parent: ParentOrGlobal::Global(realm.global.clone()),
            variables,
            available_labels: Vec::new(),
            last_label_is_current: false,
            state: ScopeState::STATE_NONE,
            this: Value::string("global"),
            file: Some(path),
        }
    }

    pub fn with_parent(parent: Gc<RefCell<Self>>) -> Result<Self> {
        let mut variables = HashMap::with_capacity(8);

        variables.insert(
            "undefined".to_string(),
            Variable::new_read_only(Value::Undefined),
        );
        variables.insert(
            "NaN".to_string(),
            Variable::new_read_only(Value::Number(f64::NAN)),
        );
        variables.insert(
            "Infinity".to_string(),
            Variable::new_read_only(Value::Number(f64::INFINITY)),
        );
        variables.insert("null".to_string(), Variable::new_read_only(Value::Null));
        variables.insert(
            "true".to_string(),
            Variable::new_read_only(Value::Boolean(true)),
        );
        variables.insert(
            "false".to_string(),
            Variable::new_read_only(Value::Boolean(false)),
        );

        let state = parent.borrow()?.state.copy();

        let par_scope = parent.borrow()?;
        let available_labels = par_scope.available_labels.clone();
        drop(par_scope);

        let this = parent.borrow()?.this.copy();

        Ok(Self {
            parent: ParentOrGlobal::Parent(parent),
            variables,
            available_labels,
            last_label_is_current: false,
            state,
            this,
            file: None,
        })
    }

    pub fn with_parent_this(parent: Gc<RefCell<Self>>, this: Value) -> Result<Self> {
        let mut new = Self::with_parent(parent)?;

        new.this = this;

        Ok(new)
    }

    pub fn declare_var(&mut self, name: String, value: Value) {
        self.variables.insert(name, Variable::new(value));
    }

    pub fn declare_read_only_var(&mut self, name: String, value: Value) -> Res {
        if self.variables.contains_key(&name) {
            return Err(Error::new("Variable already declared"));
        }

        self.variables.insert(name, Variable::new_read_only(value));

        Ok(())
    }

    pub fn declare_global_var(&mut self, name: String, value: Value) -> Res {
        #[allow(clippy::if_same_then_else)]
        if self.state.is_function() {
            self.variables.insert(name, Variable::new(value));
        } else {
            match &self.parent {
                ParentOrGlobal::Global(global) => {
                    global.define_variable(name.into(), Variable::new(value))?;
                }
                ParentOrGlobal::Parent(p) => {
                    p.borrow_mut()?.declare_global_var(name, value.copy())?;
                }
            }
        }

        Ok(())
    }

    pub fn resolve(&self, name: &str, realm: &mut Realm) -> Result<Option<Value>> {
        if let Some(v) = self.variables.get(name) {
            return Ok(Some(v.copy()));
        }

        match &self.parent {
            ParentOrGlobal::Parent(parent) => parent.borrow()?.resolve(name, realm),
            ParentOrGlobal::Global(global) => global.resolve_property(&name.into(), realm),
        }
    }

    pub fn has_value(&self, name: &str) -> Result<bool> {
        if self.variables.contains_key(name) {
            Ok(true)
        } else {
            match &self.parent {
                ParentOrGlobal::Parent(parent) => parent.borrow()?.has_value(name),
                ParentOrGlobal::Global(global) => global.contains_key(&name.into()),
            }
        }
    }

    #[must_use]
    pub fn has_label(&self, label: &str) -> bool {
        self.available_labels.contains(&label.to_string())
    }

    pub fn declare_label(&mut self, label: String) {
        self.available_labels.push(label);
        self.last_label_is_current = true;
    }

    pub fn last_label(&mut self) -> Option<&String> {
        if !self.last_label_is_current {
            return None;
        }
        self.available_labels.last()
    }

    pub fn set_no_label(&mut self) {
        self.last_label_is_current = false;
    }

    pub fn state_set_function(&mut self) {
        self.state.set_function();
    }

    pub fn state_set_iteration(&mut self) {
        self.state.set_iteration();
    }

    pub fn state_set_breakable(&mut self) {
        self.state.set_breakable();
    }

    pub fn state_set_returnable(&mut self) {
        self.state.set_returnable();
    }

    pub fn state_set_loop(&mut self) {
        self.state.set_loop();
    }

    pub fn state_set_opt_chain(&mut self) {
        self.state.set_opt_chain();
    }

    #[must_use]
    pub const fn state_is_function(&self) -> bool {
        self.state.is_function()
    }

    #[must_use]
    pub const fn state_is_iteration(&self) -> bool {
        self.state.is_iteration()
    }

    #[must_use]
    pub const fn state_is_breakable(&self) -> bool {
        self.state.is_breakable()
    }

    #[must_use]
    pub const fn state_is_returnable(&self) -> bool {
        self.state.is_returnable()
    }

    #[must_use]
    pub const fn state_is_none(&self) -> bool {
        self.state.is_none()
    }

    #[must_use]
    pub const fn state_is_continuable(&self) -> bool {
        self.state.is_continuable()
    }

    #[must_use]
    pub const fn state_is_opt_chain(&self) -> bool {
        self.state.is_opt_chain()
    }

    pub fn update(&mut self, name: &str, value: Value) -> Result<bool> {
        if let Some(v) = self.variables.get_mut(name) {
            if !v.properties.is_writable() {
                return Ok(false);
            }
            v.value = value;
            return Ok(true);
        }

        match &self.parent {
            ParentOrGlobal::Parent(p) => {
                return p.borrow_mut()?.update(name, value);
            }
            ParentOrGlobal::Global(global) => {
                let name = name.into();

                if global.contains_key(&name)? {
                    global.define_property(name, value)?;

                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    pub fn update_or_define(&mut self, name: String, value: Value) -> Res {
        if let Some(v) = self.variables.get_mut(&name) {
            if !v.properties.is_writable() {
                return Err(Error::ty("Assignment to constant variable"));
            }

            v.value = value;
            return Ok(());
        }

        match &self.parent {
            ParentOrGlobal::Parent(p) => {
                if p.borrow_mut()?.update(&name, value.copy())? {
                    return Ok(());
                }
            }
            ParentOrGlobal::Global(global) => {
                let name = name.clone().into();

                if global.contains_key(&name)? {
                    global.define_property(name, value)?;

                    return Ok(());
                }
            }
        }

        self.declare_var(name, value);

        Ok(())
    }

    pub fn set_file(&mut self, file: PathBuf) {
        self.file = Some(file);
    }

    pub fn get_current_file(&self) -> Result<PathBuf> {
        if let Some(f) = self.file.clone() {
            return Ok(f);
        }

        match &self.parent {
            ParentOrGlobal::Parent(p) => p.borrow()?.get_current_file(),
            ParentOrGlobal::Global(_) => Ok(PathBuf::new()),
        }
    }
}

impl Scope {
    #[must_use]
    pub fn new(realm: &Realm, path: PathBuf) -> Self {
        Self {
            scope: Gc::new(RefCell::new(ScopeInternal::new(realm, path))),
        }
    }

    #[must_use]
    pub fn global(realm: &Realm, path: PathBuf) -> Self {
        Self {
            scope: Gc::new(RefCell::new(ScopeInternal::global(realm, path))),
        }
    }

    pub fn with_parent(parent: &Self) -> Result<Self> {
        Ok(Self {
            scope: Gc::new(RefCell::new(ScopeInternal::with_parent(Gc::clone(
                &parent.scope,
            ))?)),
        })
    }

    pub fn with_parent_this(parent: &Self, this: Value) -> Result<Self> {
        Ok(Self {
            scope: Gc::new(RefCell::new(ScopeInternal::with_parent_this(
                Gc::clone(&parent.scope),
                this,
            )?)),
        })
    }

    pub fn declare_var(&mut self, name: String, value: Value) -> Res {
        self.scope.borrow_mut()?.declare_var(name, value);

        Ok(())
    }

    pub fn declare_read_only_var(&mut self, name: String, value: Value) -> Res {
        self.scope.borrow_mut()?.declare_read_only_var(name, value)
    }

    pub fn declare_global_var(&mut self, name: String, value: Value) -> Res {
        self.scope.borrow_mut()?.declare_global_var(name, value)?;
        Ok(())
    }

    pub fn resolve(&self, name: &str, realm: &mut Realm) -> Result<Option<Value>> {
        self.scope.borrow()?.resolve(name, realm)
    }

    pub fn has_label(&self, label: &str) -> Result<bool> {
        let Ok(scope) = self.scope.borrow() else {
            return Ok(false);
        };

        if scope.has_label(label) {
            Ok(true)
        } else {
            if let ParentOrGlobal::Parent(p) = &scope.parent {
                return Ok(p.borrow()?.has_label(label));
            }
            Ok(false)
        }
    }

    pub fn declare_label(&mut self, label: String) -> Res {
        self.scope.borrow_mut()?.declare_label(label);
        Ok(())
    }

    pub fn last_label(&self) -> Result<Option<String>> {
        Ok(self.scope.borrow_mut()?.last_label().cloned())
    }

    pub fn set_no_label(&self) -> Res {
        self.scope.borrow_mut()?.set_no_label();
        
        Ok(())
    }

    #[must_use]
    pub fn state(&self) -> ScopeState {
        self.scope
            .borrow()
            .map(|x| x.state.clone())
            .unwrap_or_default()
    }

    pub fn state_set_function(&mut self) -> Res {
        self.scope.borrow_mut()?.state_set_function();
        Ok(())
    }

    pub fn state_set_iteration(&mut self) -> Res {
        self.scope.borrow_mut()?.state_set_iteration();
        Ok(())
    }

    pub fn state_set_breakable(&mut self) -> Res {
        self.scope.borrow_mut()?.state_set_breakable();
        Ok(())
    }

    pub fn state_set_returnable(&mut self) -> Res {
        self.scope.borrow_mut()?.state_set_returnable();
        Ok(())
    }

    pub fn state_set_loop(&mut self) -> Res {
        self.scope.borrow_mut()?.state_set_loop();
        Ok(())
    }

    pub fn state_set_opt_chain(&mut self) -> Res {
        self.scope.borrow_mut()?.state_set_opt_chain();
        Ok(())
    }

    pub fn state_is_function(&self) -> Result<bool> {
        Ok(self.scope.borrow()?.state_is_function())
    }

    pub fn is_global(&self) -> Result<bool> {
        Ok(matches!(self.parent()?, ParentOrGlobal::Global(_)))
    }

    pub fn state_is_iteration(&self) -> Result<bool> {
        Ok(self.scope.borrow()?.state_is_iteration())
    }

    pub fn state_is_breakable(&self) -> Result<bool> {
        Ok(self.scope.borrow()?.state_is_breakable())
    }

    pub fn state_is_returnable(&self) -> Result<bool> {
        Ok(self.scope.borrow()?.state_is_returnable())
    }

    pub fn state_is_none(&self) -> Result<bool> {
        Ok(self.scope.borrow()?.state_is_none())
    }

    pub fn state_is_continuable(&self) -> Result<bool> {
        Ok(self.scope.borrow()?.state_is_continuable())
    }

    pub fn state_is_opt_chain(&self) -> Result<bool> {
        Ok(self.scope.borrow()?.state_is_opt_chain())
    }

    pub fn has_value(&self, name: &str) -> Result<bool> {
        self.scope.borrow()?.has_value(name)
    }

    pub fn update(&mut self, name: &str, value: Value) -> Result<bool> {
        self.scope.borrow_mut()?.update(name, value)
    }

    pub fn update_or_define(&mut self, name: String, value: Value) -> Res {
        self.scope.borrow_mut()?.update_or_define(name, value)
    }

    pub fn child(&self) -> Result<Self> {
        Self::with_parent(self)
    }

    pub fn this(&self) -> Result<Value> {
        Ok(self.scope.borrow()?.this.copy())
    }

    pub fn parent(&self) -> Result<ParentOrGlobal> {
        Ok(self.scope.borrow()?.parent.clone())
    }

    pub fn set_path(&mut self, path: PathBuf) -> Res {
        self.scope.borrow_mut()?.set_file(path);

        Ok(())
    }

    pub fn get_current_path(&self) -> Result<PathBuf> {
        self.scope.borrow()?.get_current_file()
    }
}

impl CustomGcRefUntyped for Scope {
    fn gc_untyped_ref<U: Collectable>(&self) -> Option<GcRef<U>> {
        Some(self.scope.get_untyped_ref())
    }
}

impl From<ScopeInternal> for Scope {
    fn from(scope: ScopeInternal) -> Self {
        Self {
            scope: Gc::new(RefCell::new(scope)),
        }
    }
}

impl From<Gc<RefCell<ScopeInternal>>> for Scope {
    fn from(scope: Gc<RefCell<ScopeInternal>>) -> Self {
        Self { scope }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_state_new_is_none() {
        let state = ScopeState::new();
        assert!(state.is_none());
    }

    #[test]
    fn scope_state_set_function_is_function() {
        let mut state = ScopeState::new();
        state.set_function();
        assert!(state.is_function());
    }

    #[test]
    fn scope_state_set_iteration_is_iteration() {
        let mut state = ScopeState::new();
        state.set_iteration();
        assert!(state.is_iteration());
    }

    #[test]
    fn scope_state_set_breakable_is_breakable() {
        let mut state = ScopeState::new();
        state.set_breakable();
        assert!(state.is_breakable());
    }

    #[test]
    fn scope_state_set_returnable_is_returnable() {
        let mut state = ScopeState::new();
        state.set_returnable();
        assert!(state.is_returnable());
    }

    #[test]
    fn scope_state_set_loop_is_continuable() {
        let mut state = ScopeState::new();
        state.set_loop();
        assert!(state.is_continuable());
    }

    #[test]
    fn scope_internal_declare_var_and_resolve() {
        let mut realm = Realm::new().unwrap();
        let mut scope = ScopeInternal::new(&realm, PathBuf::from("test.js"));
        scope.declare_var("test".to_string(), Value::Number(42.0));
        let value = scope.resolve("test", &mut realm).unwrap().unwrap();
        assert_eq!(value, Value::Number(42.0));
    }

    #[test]
    fn scope_internal_declare_read_only_var_and_update_fails() {
        let realm = Realm::new().unwrap();
        let mut scope = ScopeInternal::new(&realm, PathBuf::from("test.js"));
        scope
            .declare_read_only_var("test".to_string(), Value::Number(42.0))
            .unwrap();
        let result = scope.update("test", Value::Number(43.0)).unwrap();
        assert!(!result);
    }

    #[test]
    fn scope_internal_declare_global_var_and_resolve() {
        let mut realm = Realm::new().unwrap();
        let mut scope = ScopeInternal::new(&realm, PathBuf::from("test.js"));
        scope
            .declare_global_var("test".to_string(), Value::Number(42.0))
            .unwrap();
        let value = scope.resolve("test", &mut realm).unwrap().unwrap();
        assert_eq!(value, Value::Number(42.0));
    }

    #[test]
    fn scope_internal_update_or_define_and_resolve() {
        let mut realm = Realm::new().unwrap();
        let mut scope = ScopeInternal::new(&realm, PathBuf::from("test.js"));
        scope
            .update_or_define("test".to_string(), Value::Number(42.0))
            .unwrap();
        let value = scope.resolve("test", &mut realm).unwrap().unwrap();
        assert_eq!(value, Value::Number(42.0));
    }
}
