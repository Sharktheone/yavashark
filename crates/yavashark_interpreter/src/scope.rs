use std::collections::HashMap;
use yavashark_value::Value;
use std::cell::RefCell;
use std::ptr::NonNull;
use std::rc::Rc;


pub struct Scope {
    scope: Rc<RefCell<ScopeInternal>>,
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
        }
    }

    pub fn with_parent(parent: &Scope) -> Self {
        Self {
            scope: Rc::new(RefCell::new(ScopeInternal::with_parent(Rc::clone(&parent.scope)))),
        }
    }

    pub fn clone(scope: &Self) -> Self {
        Self {
            scope: Rc::clone(&scope.scope),
        }
    }
}