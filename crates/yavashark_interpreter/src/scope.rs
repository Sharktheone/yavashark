use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use yavashark_value::Value;


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
    pub available_labels: Vec<String>,
    pub state: ScopeState,
}


struct ScopeInternal {
    parent: Option<Rc<RefCell<ScopeInternal>>>,
    variables: HashMap<String, Value>,
}

impl ScopeInternal {
    pub fn new() -> Self {
        Self {
            parent: None,
            variables: HashMap::new(),
        }
    }

    pub fn with_parent(parent: Rc<RefCell<ScopeInternal>>) -> Self {
        Self {
            parent: Some(parent),
            variables: HashMap::new(),
        }
    }
}


impl Scope {
    pub fn new() -> Self {
        Self {
            scope: Rc::new(RefCell::new(ScopeInternal::new())),
            available_labels: Vec::new(),
            state: ScopeState::STATE_NONE, 
        }
    }

    pub fn with_parent(parent: &Scope) -> Self {
        Self {
            scope: Rc::new(RefCell::new(ScopeInternal::with_parent(Rc::clone(&parent.scope)))),
            available_labels: Vec::new(),
            state: ScopeState::clone(&parent.state),
        }
    }
}