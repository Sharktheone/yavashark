use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::Res;

use crate::variable::Variable;

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
    const STATE_NONE: ScopeState = ScopeState { state: ScopeState::NONE };
    const STATE_GLOBAL: ScopeState = ScopeState { state: ScopeState::GLOBAL };
    const STATE_FUNCTION: ScopeState = ScopeState { state: ScopeState::FUNCTION };
    const STATE_ITERATION: ScopeState = ScopeState { state: ScopeState::ITERATION };
    const STATE_BREAKABLE: ScopeState = ScopeState { state: ScopeState::BREAKABLE };
    const STATE_RETURNABLE: ScopeState = ScopeState { state: ScopeState::RETURNABLE };

    pub fn new() -> Self {
        Self {
            state: 0,
        }
    }

    pub fn clone(state: &Self) -> Self {
        let mut state = state.state;


        state &= !Self::FUNCTION; // Remove the function state
        state &= !Self::GLOBAL; // Remove the global state

        Self {
            state,
        }
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

    pub fn is_function(&self) -> bool {
        self.state & Self::FUNCTION != 0
    }

    pub fn is_global(&self) -> bool {
        self.state & Self::GLOBAL != 0
    }

    pub fn is_iteration(&self) -> bool {
        self.state & Self::ITERATION != 0
    }

    pub fn is_breakable(&self) -> bool {
        self.state & Self::BREAKABLE != 0
    }

    pub fn is_returnable(&self) -> bool {
        self.state & Self::RETURNABLE != 0
    }

    pub fn is_none(&self) -> bool {
        self.state == Self::NONE
    }
}


pub struct Scope {
    scope: Rc<RefCell<ScopeInternal>>,
}


struct ScopeInternal {
    parent: Option<Rc<RefCell<ScopeInternal>>>,
    variables: HashMap<String, Variable>,
    pub available_labels: Vec<String>,
    pub state: ScopeState,
}

impl ScopeInternal {
    pub fn new() -> Self {
        let mut variables = HashMap::with_capacity(8);

        variables.insert("undefined".to_string(), Variable::new_read_only(Value::Undefined));
        variables.insert("NaN".to_string(), Variable::new_read_only(Value::Number(f64::NAN)));
        variables.insert("Infinity".to_string(), Variable::new_read_only(Value::Number(f64::INFINITY)));
        variables.insert("null".to_string(), Variable::new_read_only(Value::Null));
        variables.insert("true".to_string(), Variable::new_read_only(Value::Boolean(true)));
        variables.insert("false".to_string(), Variable::new_read_only(Value::Boolean(false)));
        Self {
            parent: None,
            variables,
            available_labels: Vec::new(),
            state: ScopeState::new(),
        }
    }
    
    
    pub fn global() -> Self {
        let mut variables = HashMap::with_capacity(8);
        
        variables.insert("undefined".to_string(), Variable::new_read_only(Value::Undefined));
        variables.insert("NaN".to_string(), Variable::new_read_only(Value::Number(f64::NAN)));
        variables.insert("Infinity".to_string(), Variable::new_read_only(Value::Number(f64::INFINITY)));
        variables.insert("null".to_string(), Variable::new_read_only(Value::Null));
        variables.insert("true".to_string(), Variable::new_read_only(Value::Boolean(true)));
        variables.insert("false".to_string(), Variable::new_read_only(Value::Boolean(false)));
        
        Self {
            parent: None,
            variables,
            available_labels: Vec::new(),
            state: ScopeState::STATE_GLOBAL,
        }
    }

    pub fn with_parent(parent: Rc<RefCell<ScopeInternal>>) -> Self {
        let mut variables = HashMap::with_capacity(8);

        variables.insert("undefined".to_string(), Variable::new_read_only(Value::Undefined));
        variables.insert("NaN".to_string(), Variable::new_read_only(Value::Number(f64::NAN)));
        variables.insert("Infinity".to_string(), Variable::new_read_only(Value::Number(f64::INFINITY)));
        variables.insert("null".to_string(), Variable::new_read_only(Value::Null));
        variables.insert("true".to_string(), Variable::new_read_only(Value::Boolean(true)));
        variables.insert("false".to_string(), Variable::new_read_only(Value::Boolean(false)));

        Self {
            parent: Some(parent),
            variables,
            available_labels: Vec::new(),
            state: ScopeState::new(),
        }
    }

    pub fn declare_var(&mut self, name: String, value: Value) {
        self.variables.insert(name, Variable::new(value));
    }

    pub fn declare_read_only_var(&mut self, name: String, value: Value) -> Res {
        if self.variables.contains_key(&name) {
            return Err(Error::new("Variable already declared".to_string()));
        }

        self.variables.insert(name, Variable::new_read_only(value));

        Ok(())
    }

    pub fn declare_global_var(&mut self, name: String, value: Value) {
        #[allow(clippy::if_same_then_else)]
        if self.state.is_global() || self.state.is_function() {
            self.variables.insert(name, Variable::new(value));
        } else if let Some(p) = self.parent.as_ref() {
            p.borrow_mut().declare_global_var(name.clone(), value.clone());
        } else  {
            self.variables.insert(name, Variable::new(value));
        }
    }
    
    pub fn resolve_var(&self, name: &str) -> Option<Value> {
        if let Some(v) = self.variables.get(name) {
            return Some(v.cloned());
        }
        
        if let Some(p) = self.parent.as_ref() {
            return p.borrow().resolve_var(name);
        }
        
        None
    }
}


impl Scope {
    pub fn new() -> Self {
        Self {
            scope: Rc::new(RefCell::new(ScopeInternal::new())),
        }
    }
    
    pub fn global() -> Self {
        Self {
            scope: Rc::new(RefCell::new(ScopeInternal::global())),
        }
    }

    pub fn with_parent(parent: &Scope) -> Self {
        Self {
            scope: Rc::new(RefCell::new(ScopeInternal::with_parent(Rc::clone(&parent.scope)))),
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
}