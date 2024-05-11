use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::{Error, Res, Value, Variable};
use crate::console::get_console;
use crate::context::Context;
use crate::error::get_error;

pub struct MutValue {
    pub(crate) name: String,
    pub(crate) scope: Rc<RefCell<ScopeInternal>>,
}

#[derive(Debug, Clone, Default)]
#[allow(clippy::module_name_repetitions)]
pub struct ScopeState {
    state: u8,
}

impl ScopeState {
    const NONE: u8 = 0b0;

    const GLOBAL: u8 = 0b1;
    const FUNCTION: u8 = 0b10;
    const ITERATION: u8 = 0b100;
    const BREAKABLE: u8 = 0b1000;
    const RETURNABLE: u8 = 0b10000;
    const CONTINUABLE: u8 = 0b10_0000;
    const STATE_NONE: Self = Self {
        state: Self::NONE,
    };
    const STATE_GLOBAL: Self = Self {
        state: Self::GLOBAL,
    };
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

    #[must_use]
    pub const fn new() -> Self {
        Self { state: 0 }
    }

    #[must_use]
    pub const fn copy(&self) -> Self {
        let mut state = self.state;

        state &= !Self::FUNCTION; // Remove the function state
        state &= !Self::GLOBAL; // Remove the global state

        Self { state }
    }

    pub fn set_global(&mut self) {
        self.state |= Self::GLOBAL;
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

    #[must_use]
    pub const fn is_function(&self) -> bool {
        self.state & Self::FUNCTION != 0
    }

    #[must_use]
    pub const fn is_global(&self) -> bool {
        self.state & Self::GLOBAL != 0
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
}

#[derive(Debug, Clone)]
pub struct Scope {
    scope: Rc<RefCell<ScopeInternal>>,
    pub this: Value,
}

#[derive(Debug)]
pub(crate) struct ScopeInternal {
    parent: Option<Rc<RefCell<ScopeInternal>>>,
    variables: HashMap<String, Variable>,
    pub available_labels: Vec<String>,
    pub state: ScopeState,
}

impl ScopeInternal {
    pub fn new(ctx: &mut Context) -> Self {
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
            Variable::new_read_only(get_console(ctx)),
        );
        Self {
            parent: None,
            variables,
            available_labels: Vec::new(),
            state: ScopeState::new(),
        }
    }

    pub fn global(ctx: &mut Context) -> Self {
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
            Variable::new_read_only(get_console(ctx)),
        );

        variables.insert("Error".to_string(), Variable::new_read_only(get_error(ctx)));

        Self {
            parent: None,
            variables,
            available_labels: Vec::new(),
            state: ScopeState::STATE_GLOBAL,
        }
    }

    pub fn with_parent(parent: Rc<RefCell<Self>>) -> Self {
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

        let state = parent.borrow().state.copy();

        let par_scope = parent.borrow();
        let mut available_labels = par_scope.available_labels.clone();
        drop(par_scope);

        Self {
            parent: Some(parent),
            variables,
            available_labels,
            state,
        }
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

    pub fn declare_global_var(&mut self, name: String, value: Value) {
        #[allow(clippy::if_same_then_else)]
        if self.state.is_global() || self.state.is_function() {
            self.variables.insert(name, Variable::new(value));
        } else if let Some(p) = self.parent.as_ref() {
            p.borrow_mut()
                .declare_global_var(name, value.copy());
        } else {
            self.variables.insert(name, Variable::new(value));
        }
    }

    pub fn resolve(&self, name: &str) -> Option<Value> {
        if let Some(v) = self.variables.get(name) {
            return Some(v.copy());
        }

        if let Some(p) = self.parent.as_ref() {
            return p.borrow().resolve(name);
        }

        None
    }

    pub fn has_value(&self, name: &str) -> bool {
        return if self.variables.contains_key(name) {
            true
        } else {
            self.parent.as_ref()
                .map_or(false, |p| p.borrow().has_value(name))
        };
    }

    pub fn has_label(&self, label: &str) -> bool {
        self.available_labels.contains(&label.to_string())
    }

    pub fn declare_label(&mut self, label: String) {
        self.available_labels.push(label);
    }

    pub fn last_label(&mut self) -> Option<&String> {
        self.available_labels.last()
    }

    pub fn state_set_global(&mut self) {
        self.state.set_global();
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

    pub const fn state_is_function(&self) -> bool {
        self.state.is_function()
    }

    pub const fn state_is_global(&self) -> bool {
        self.state.is_global()
    }

    pub const fn state_is_iteration(&self) -> bool {
        self.state.is_iteration()
    }

    pub const fn state_is_breakable(&self) -> bool {
        self.state.is_breakable()
    }

    pub const fn state_is_returnable(&self) -> bool {
        self.state.is_returnable()
    }

    pub const fn state_is_none(&self) -> bool {
        self.state.is_none()
    }

    pub const fn state_is_continuable(&self) -> bool {
        self.state.is_continuable()
    }

    pub fn update(&mut self, name: &str, value: Value) -> bool {
        if let Some(v) = self.variables.get_mut(name) {
            if !v.properties.is_writable() {
                return false;
            }
            v.value = value;
            return true;
        } else if let Some(p) = self.parent.as_ref() {
            return p.borrow_mut().update(name, value);
        }

        false
    }

    pub fn update_or_define(&mut self, name: String, value: Value) -> Res {
        if let Some(v) = self.variables.get_mut(&name) {
            if !v.properties.is_writable() {
                return Err(Error::ty("Assignment to constant variable"));
            }

            v.value = value;
            return Ok(());
        } else if let Some(p) = self.parent.as_ref() {
            if p.borrow_mut().update(&name, value.copy()) {
                return Ok(());
            }
        }

        self.declare_var(name, value);

        Ok(())
    }

    pub fn with_mut(&mut self, name: &str, f: &impl Fn(&mut Value)) -> Res {
        if let Some(v) = self.variables.get_mut(name) {
            if !v.properties.is_writable() {
                return Err(Error::ty("Assignment to constant variable"));
            }

            f(&mut v.value);
            Ok(())
        } else if let Some(p) = self.parent.as_ref() {
            p.borrow_mut().with_mut(name, f)
        } else {
            Err(Error::new("Variable not found"))
        }
    }

    pub fn with(&self, name: &str, f: &impl Fn(&Value)) -> Res {
        self.variables.get(name)
            .map_or_else(|| self.parent.as_ref()
                .map_or_else(|| Err(Error::new("Variable not found")), |p| p.borrow().with(name, f)), |v| {
                f(&v.value);
                Ok(())
            })
    }
}

impl Scope {
    pub fn new(ctx: &mut Context) -> Self {
        Self {
            scope: Rc::new(RefCell::new(ScopeInternal::new(ctx))),
            this: Value::Undefined,
        }
    }

    pub fn global(ctx: &mut Context) -> Self {
        Self {
            scope: Rc::new(RefCell::new(ScopeInternal::global(ctx))),
            this: Value::String("global".to_string()), //TODO: globalThis
        }
    }

    #[must_use]
    pub fn with_parent(parent: &Self) -> Self {
        Self {
            scope: Rc::new(RefCell::new(ScopeInternal::with_parent(Rc::clone(
                &parent.scope,
            )))),
            this: parent.this.copy(),
        }
    }

    #[must_use]
    pub fn with_parent_this(parent: &Self, this: Value) -> Self {
        Self {
            scope: Rc::new(RefCell::new(ScopeInternal::with_parent(Rc::clone(
                &parent.scope,
            )))),
            this,
        }
    }

    pub fn declare_var(&mut self, name: String, value: Value) {
        self.scope.borrow_mut().declare_var(name, value);
    }

    pub fn declare_read_only_var(&mut self, name: String, value: Value) -> Res {
        self.scope.borrow_mut().declare_read_only_var(name, value)
    }

    pub fn declare_global_var(&mut self, name: String, value: Value) {
        self.scope.borrow_mut().declare_global_var(name, value);
    }

    #[must_use]
    pub fn resolve(&self, name: &str) -> Option<Value> {
        self.scope.borrow().resolve(name)
    }

    #[must_use]
    pub fn has_label(&self, label: &str) -> bool {
        let scope = self.scope.borrow();
        if scope.has_label(label) {
            true
        } else {
            if let Some(p) = scope.parent.as_ref() {
                return p.borrow().has_label(label);
            }
            false
        }
    }

    pub fn declare_label(&mut self, label: String) {
        self.scope.borrow_mut().declare_label(label);
    }

    #[must_use]
    pub fn last_label(&self) -> Option<String> {
        self.scope.borrow_mut().last_label().cloned()
    }

    #[must_use]
    pub fn state(&self) -> ScopeState {
        self.scope.borrow().state.clone()
    }

    pub fn state_set_global(&mut self) {
        self.scope.borrow_mut().state_set_global();
    }

    pub fn state_set_function(&mut self) {
        self.scope.borrow_mut().state_set_function();
    }

    pub fn state_set_iteration(&mut self) {
        self.scope.borrow_mut().state_set_iteration();
    }

    pub fn state_set_breakable(&mut self) {
        self.scope.borrow_mut().state_set_breakable();
    }

    pub fn state_set_returnable(&mut self) {
        self.scope.borrow_mut().state_set_returnable();
    }

    pub fn state_set_loop(&mut self) {
        self.scope.borrow_mut().state_set_loop();
    }

    #[must_use]
    pub fn state_is_function(&self) -> bool {
        self.scope.borrow().state_is_function()
    }

    #[must_use]
    pub fn state_is_global(&self) -> bool {
        self.scope.borrow().state_is_global()
    }

    #[must_use]
    pub fn state_is_iteration(&self) -> bool {
        self.scope.borrow().state_is_iteration()
    }

    #[must_use]
    pub fn state_is_breakable(&self) -> bool {
        self.scope.borrow().state_is_breakable()
    }

    #[must_use]
    pub fn state_is_returnable(&self) -> bool {
        self.scope.borrow().state_is_returnable()
    }

    #[must_use]
    pub fn state_is_none(&self) -> bool {
        self.scope.borrow().state_is_none()
    }

    #[must_use]
    pub fn state_is_continuable(&self) -> bool {
        self.scope.borrow().state_is_continuable()
    }

    #[must_use]
    pub fn has_value(&self, name: &str) -> bool {
        self.scope.borrow().has_value(name)
    }

    pub fn update(&mut self, name: &str, value: Value) -> bool {
        self.scope.borrow_mut().update(name, value)
    }

    pub fn update_or_define(&mut self, name: String, value: Value) -> Res {
        self.scope.borrow_mut().update_or_define(name, value)
    }

    pub fn with_mut(&mut self, name: &str, f: &impl Fn(&mut Value)) -> Res {
        self.scope.borrow_mut().with_mut(name, f)
    }

    pub fn with(&self, name: &str, f: &impl Fn(&Value)) -> Res {
        self.scope.borrow().with(name, f)
    }

    #[must_use]
    pub fn child(&self) -> Self {
        Self::with_parent(self)
    }
}

impl From<ScopeInternal> for Scope {
    fn from(scope: ScopeInternal) -> Self {
        Self {
            scope: Rc::new(RefCell::new(scope)),
            this: Value::Undefined,
        }
    }
}

impl From<Rc<RefCell<ScopeInternal>>> for Scope {
    fn from(scope: Rc<RefCell<ScopeInternal>>) -> Self {
        Self {
            scope,
            this: Value::Undefined,
        }
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
    fn scope_state_set_global_is_global() {
        let mut state = ScopeState::new();
        state.set_global();
        assert!(state.is_global());
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
        let mut ctx = Context::new().unwrap();
        let mut scope = ScopeInternal::new(&mut ctx);
        scope.declare_var("test".to_string(), Value::Number(42.0));
        let value = scope.resolve("test").unwrap();
        assert_eq!(value, Value::Number(42.0));
    }

    #[test]
    fn scope_internal_declare_read_only_var_and_update_fails() {
        let mut ctx = Context::new().unwrap();
        let mut scope = ScopeInternal::new(&mut ctx);
        scope
            .declare_read_only_var("test".to_string(), Value::Number(42.0))
            .unwrap();
        let result = scope.update("test", Value::Number(43.0));
        assert!(!result);
    }

    #[test]
    fn scope_internal_declare_global_var_and_resolve() {
        let mut ctx = Context::new().unwrap();
        let mut scope = ScopeInternal::new(&mut ctx);
        scope.declare_global_var("test".to_string(), Value::Number(42.0));
        let value = scope.resolve("test").unwrap();
        assert_eq!(value, Value::Number(42.0));
    }

    #[test]
    fn scope_internal_update_or_define_and_resolve() {
        let mut ctx = Context::new().unwrap();
        let mut scope = ScopeInternal::new(&mut ctx);
        scope
            .update_or_define("test".to_string(), Value::Number(42.0))
            .unwrap();
        let value = scope.resolve("test").unwrap();
        assert_eq!(value, Value::Number(42.0));
    }
}
