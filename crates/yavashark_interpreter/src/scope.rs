use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use yavashark_value::Value;

pub struct Scope {
    scope: Rc<RefCell<ScopeInternal>>,
    pub in_iter: bool,
    pub in_function: bool,
    pub available_labels: Vec<String>,
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
            in_iter: false,
            in_function: false,
            available_labels: Vec::new(),
        }
    }

    pub fn with_parent(parent: &Scope) -> Self {
        Self {
            scope: Rc::new(RefCell::new(ScopeInternal::with_parent(Rc::clone(&parent.scope)))),
            in_iter: false,
            in_function: false,
            available_labels: Vec::new(),
        }
    }

    pub fn clone(scope: &Self) -> Self {
        Self {
            scope: Rc::clone(&scope.scope),
            in_iter: scope.in_iter,
            in_function: scope.in_function,
            available_labels: scope.available_labels.clone(),
        }
    }
}